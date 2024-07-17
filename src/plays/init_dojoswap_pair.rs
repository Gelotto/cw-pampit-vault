use cosmwasm_std::{
    coin, to_json_binary, Addr, Coin, DepsMut, Env, Reply, Response, StdError, Storage, SubMsg, SubMsgResult, WasmMsg,
};

use crate::{
    error::ContractError,
    integrations::dojoswap::{
        DojoswapAsset, DojoswapAssetInfo, DojoswapFactoryExecuteMsg, DojoswapFactoryQueryMsg, DojoswapPairInfo,
        DOJOSWAP_PAIR_FACTORY_ADDR,
    },
    math::add_u64,
    msg::InstantiateMsg,
    state::{
        models::{DojoswapCreatePairState, ReplyHandler},
        storage::{DOJOSWAP_PAIR_ADDR, REPLY_HANDLERS, REPLY_ID_COUNTER},
    },
    tokens::Token,
};

use super::utils::{prepare_pair_token_amounts, PairAmounts};

pub fn init_dojoswap_pair(
    store: &mut dyn Storage,
    resp: &mut Response,
    msg: &InstantiateMsg,
) -> Result<Response, ContractError> {
    let InstantiateMsg {
        vl,
        quote,
        base,
        fee_recipient,
        ..
    } = msg;

    let PairAmounts {
        quote: q_pair_amount,
        base: b_pair_amount,
        fee: q_fee_amount,
    } = prepare_pair_token_amounts(quote.amount, base.amount, *vl)?;

    let mut funds: Vec<Coin> = Vec::with_capacity(2);

    match &quote.token {
        Token::Denom(denom) => funds.push(coin(q_pair_amount.u128(), denom.to_owned())),
        _ => {
            return Err(ContractError::NotImplemented {
                reason: "cw20 tokens not supported for dojoswap pool initialization".to_owned(),
            });
        },
    }

    match &base.token {
        Token::Denom(denom) => funds.push(coin(b_pair_amount.u128(), denom.to_owned())),
        _ => {
            return Err(ContractError::NotImplemented {
                reason: "cw20 tokens not supported for dojoswap pool initialization".to_owned(),
            });
        },
    }

    let assets = [
        DojoswapAsset {
            amount: q_pair_amount,
            info: DojoswapAssetInfo::NativeToken {
                denom: quote.token.get_denom().unwrap(),
            },
        },
        DojoswapAsset {
            amount: b_pair_amount,
            info: DojoswapAssetInfo::NativeToken {
                denom: base.token.get_denom().unwrap(),
            },
        },
    ];

    // Send a submsg to the factory to create new pair
    let reply_id = REPLY_ID_COUNTER
        .update(store, |n| -> Result<_, ContractError> { add_u64(n, 1u64) })?
        .u64();

    REPLY_HANDLERS.save(
        store,
        reply_id,
        &&ReplyHandler::DojoswapCreatePair(DojoswapCreatePairState {
            asset_infos: vec![assets[0].info.to_owned(), assets[1].info.to_owned()],
        }),
    )?;

    Ok(resp
        .to_owned()
        .add_submessage(quote.token.transfer(&fee_recipient, q_fee_amount)?)
        .add_submessage(SubMsg::reply_on_success(
            WasmMsg::Execute {
                contract_addr: DOJOSWAP_PAIR_FACTORY_ADDR.to_owned(),
                msg: to_json_binary(&DojoswapFactoryExecuteMsg::CreatePair {
                    assets: assets.to_owned(),
                })?,
                funds,
            },
            reply_id,
        )))
}

pub fn on_dojoswap_create_pair_reply(
    deps: DepsMut,
    _env: Env,
    reply: Reply,
    state: DojoswapCreatePairState,
) -> Result<Response, ContractError> {
    let DojoswapCreatePairState { asset_infos } = state;
    if let SubMsgResult::Ok(_) = &reply.result {
        let DojoswapPairInfo { contract_addr, .. } = deps.querier.query_wasm_smart(
            DOJOSWAP_PAIR_FACTORY_ADDR,
            &DojoswapFactoryQueryMsg::Pair {
                asset_infos: [asset_infos[0].to_owned(), asset_infos[1].to_owned()],
            },
        )?;
        DOJOSWAP_PAIR_ADDR.save(deps.storage, &Addr::unchecked(contract_addr))?;
    } else {
        return Err(ContractError::Std(StdError::generic_err(
            "failed to extract dojoswap pair address",
        )));
    }

    Ok(Response::new())
}
