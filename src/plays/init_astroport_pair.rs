use astroport::{
    asset::{Asset, AssetInfo, PairInfo},
    factory::{ExecuteMsg as AstroportFactoryExecuteMsg, PairType, QueryMsg as AstroportFactoryQueryMsg},
    pair::ExecuteMsg as AstroportPairExecuteMsg,
};
use cosmwasm_std::{
    coin, to_json_binary, Addr, BankMsg, Coin, DepsMut, Env, Reply, Response, StdError, Storage, SubMsg, SubMsgResult,
    Uint128, WasmMsg,
};

use crate::{
    error::ContractError,
    integrations::astroport::STARGAZE_ASTROPORT_FACTORY_ADDR,
    math::{add_u64, mul_ratio_u128},
    msg::InstantiateMsg,
    state::{
        models::{AstroportCreatePairState, ReplyHandler},
        storage::{ASTROPORT_PAIR_ADDR, REPLY_HANDLERS, REPLY_ID_COUNTER},
    },
    tokens::TokenAmount,
};

use super::utils::{prepare_pair_token_amounts, PairAmounts};

pub fn init_astroport_pair(
    store: &mut dyn Storage,
    resp: &mut Response,
    msg: &InstantiateMsg,
) -> Result<Response, ContractError> {
    // Compute amoutn to send into pair such that the open price on the DEX is
    // the same as the closing price on PampIt.
    let InstantiateMsg {
        vl,
        quote,
        base,
        fee_recipient,
        ..
    } = msg;

    let PairAmounts {
        base: mut pair_base_amount,
        quote: mut pair_quote_amount,
        fee: fee_amount,
    } = prepare_pair_token_amounts(quote.amount, base.amount, *vl)?;

    let lp_token_creation_fee = Uint128::from(100_000_000u128);

    // Adjust amounts after subtracting fee for astroport's LP token creation
    pair_quote_amount -= lp_token_creation_fee;
    pair_base_amount = mul_ratio_u128(
        pair_base_amount,
        pair_quote_amount,
        pair_quote_amount + lp_token_creation_fee,
    )?;

    Ok(resp
        .to_owned()
        .add_submessage(create_pair(
            store,
            TokenAmount {
                token: base.token.to_owned(),
                amount: pair_base_amount,
            },
            TokenAmount {
                token: quote.token.to_owned(),
                amount: pair_quote_amount,
            },
            lp_token_creation_fee,
        )?)
        .add_submessage(SubMsg::new(BankMsg::Send {
            to_address: fee_recipient.to_owned().into_string(),
            amount: vec![coin(fee_amount.u128(), quote.token.get_denom().unwrap())],
        })))
}

/// Check if a pair contract exists between the given two token types. If it
/// does, simply set the contract's address in the values map; otherwise,
/// instantiate a new pair via the astroport factory and store the address in
/// the reply handler.
pub fn create_pair(
    store: &mut dyn Storage,
    base: TokenAmount,
    quote: TokenAmount,
    lp_fee: Uint128,
) -> Result<SubMsg, ContractError> {
    // Prepare asset info for new pair. Note that astroport requires assets to
    // be sorted alphabetically by denom.
    let mut denoms = vec![base.token.get_denom().unwrap(), quote.token.get_denom().unwrap()];
    denoms.sort();

    let asset_infos = vec![
        AssetInfo::NativeToken {
            denom: denoms[0].to_owned(),
        },
        AssetInfo::NativeToken {
            denom: denoms[1].to_owned(),
        },
    ];

    // Send a submsg to the factory to create new pair
    let reply_id = REPLY_ID_COUNTER
        .update(store, |n| -> Result<_, ContractError> { add_u64(n, 1u64) })?
        .u64();

    let base_denom = base.token.get_denom().unwrap();
    let quote_denom = quote.token.get_denom().unwrap();

    REPLY_HANDLERS.save(
        store,
        reply_id,
        &ReplyHandler::AstroportCreatePair(AstroportCreatePairState {
            assets: vec![
                Asset {
                    amount: base.amount,
                    info: AssetInfo::NativeToken {
                        denom: base_denom.to_owned(),
                    },
                },
                Asset {
                    amount: quote.amount,
                    info: AssetInfo::NativeToken {
                        denom: quote_denom.to_owned(),
                    },
                },
            ],
            funds: vec![
                coin(base.amount.u128(), base_denom),
                coin(quote.amount.u128(), quote_denom.to_owned()),
            ],
        }),
    )?;

    let create_pair_submsg: SubMsg<_> = SubMsg::reply_on_success(
        WasmMsg::Execute {
            funds: vec![coin(lp_fee.u128(), quote_denom)],
            contract_addr: STARGAZE_ASTROPORT_FACTORY_ADDR.to_string(),
            msg: to_json_binary(&AstroportFactoryExecuteMsg::CreatePair {
                pair_type: PairType::Xyk {},
                asset_infos: asset_infos.to_owned(),
                init_params: None,
            })?,
        },
        reply_id,
    );

    Ok(create_pair_submsg)
}

pub fn provide_liquidity_to_astroport_pair(
    vault_contract_addr: Addr,
    pair_addr: String,
    assets: Vec<Asset>,
    funds: Vec<Coin>,
) -> Result<SubMsg, ContractError> {
    // Build submsg to add liquidity
    let provide_liquidity_submsg: SubMsg<_> = SubMsg::new(WasmMsg::Execute {
        funds,
        contract_addr: pair_addr.to_string(),
        msg: to_json_binary(&AstroportPairExecuteMsg::ProvideLiquidity {
            assets,
            auto_stake: Some(false),
            receiver: Some(vault_contract_addr.to_string()),
            min_lp_to_receive: None,
            slippage_tolerance: None,
        })?,
    });

    Ok(provide_liquidity_submsg)
}

/// Extract and store Pair contract address, used in submsg reply.
pub fn on_astroport_create_pair_reply(
    deps: DepsMut,
    env: Env,
    reply: Reply,
    state: AstroportCreatePairState,
) -> Result<Response, ContractError> {
    // Extract pair contract addr
    let AstroportCreatePairState { assets, funds } = state;
    let mut pair_contract_addr: Box<String> = Box::new("".to_owned());
    match reply.result {
        SubMsgResult::Ok(_) => {
            let PairInfo { contract_addr, .. } = deps.querier.query_wasm_smart(
                STARGAZE_ASTROPORT_FACTORY_ADDR,
                &AstroportFactoryQueryMsg::Pair {
                    asset_infos: vec![assets[0].info.to_owned(), assets[1].info.to_owned()],
                },
            )?;
            *pair_contract_addr = contract_addr.to_string();
        },
        SubMsgResult::Err(e) => {
            return Err(ContractError::Std(StdError::generic_err(e.to_string())));
        },
    }

    ASTROPORT_PAIR_ADDR.save(deps.storage, &Addr::unchecked(*pair_contract_addr.to_owned()))?;
    REPLY_HANDLERS.remove(deps.storage, reply.id);

    Ok(Response::new().add_submessage(provide_liquidity_to_astroport_pair(
        env.contract.address,
        *pair_contract_addr,
        assets,
        funds,
    )?))
}
