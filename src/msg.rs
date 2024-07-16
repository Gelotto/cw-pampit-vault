use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

use crate::{state::models::Config, tokens::TokenAmount};

pub const PLAY_INIT_ASTROPORT_PAIR: &str = "init_astroport_pair";
pub const PLAY_INIT_DOJOSWAP_PAIR: &str = "init_dojoswap_pair";

#[cw_serde]
pub struct InstantiateMsg {
    pub vl: Uint128,
    pub quote: TokenAmount,
    pub base: TokenAmount,
    pub manager: Addr,
    pub fee_recipient: Addr,
    pub play: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    SetConfig(Config),
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct ConfigResponse(pub Config);
