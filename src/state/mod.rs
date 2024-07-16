pub mod models;
pub mod storage;

use cosmwasm_std::{Response, Uint64};
use models::Config;
use storage::{
    CONFIG, CREATED_AT, CREATED_BY, INITIAL_BASE_TOKEN_AMOUNT, INITIAL_PLAY, INITIAL_QUOTE_TOKEN_AMOUNT,
    INITIAL_VIRTIAL_LIQUIDITY, MANAGER, REPLY_ID_COUNTER,
};

use crate::{
    error::ContractError,
    execute::Context,
    msg::{InstantiateMsg, PLAY_INIT_ASTROPORT_PAIR, PLAY_INIT_DOJOSWAP_PAIR},
    plays::{init_astroport_pair::init_astroport_pair, init_dojoswap_pair::init_dojoswap_pair},
};

pub const PLATFORM_FEE_PCT: u128 = 10_000;

/// Top-level initialization of contract state
pub fn init(
    ctx: Context,
    msg: &InstantiateMsg,
) -> Result<Response, ContractError> {
    let Context { deps, env, info } = ctx;

    CONFIG.save(deps.storage, &Config {})?;
    MANAGER.save(deps.storage, &deps.api.addr_validate(msg.manager.as_str())?)?;
    CREATED_BY.save(deps.storage, &info.sender)?;
    CREATED_AT.save(deps.storage, &env.block.time)?;
    INITIAL_PLAY.save(deps.storage, &msg.play)?;
    INITIAL_BASE_TOKEN_AMOUNT.save(deps.storage, &msg.base)?;
    INITIAL_QUOTE_TOKEN_AMOUNT.save(deps.storage, &msg.quote)?;
    INITIAL_VIRTIAL_LIQUIDITY.save(deps.storage, &msg.vl)?;
    REPLY_ID_COUNTER.save(deps.storage, &Uint64::zero())?;

    let mut resp = Response::new().add_attribute("action", "instantiate");

    match msg.play.as_str() {
        PLAY_INIT_ASTROPORT_PAIR => init_astroport_pair(deps.storage, &mut resp, msg),
        PLAY_INIT_DOJOSWAP_PAIR => init_dojoswap_pair(deps.storage, &mut resp, msg),
        _ => {
            return Err(ContractError::ValidationError {
                reason: format!("unrecognized play: {}", msg.play),
            })
        },
    }
}
