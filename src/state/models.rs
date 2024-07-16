use astroport::asset::Asset;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, DepsMut, Env, Reply, Response};

use crate::{
    error::ContractError,
    integrations::dojoswap::DojoswapAssetInfo,
    plays::{init_astroport_pair::on_astroport_create_pair_reply, init_dojoswap_pair::on_dojoswap_create_pair_reply},
};

#[cw_serde]
pub struct Config {}

#[cw_serde]
pub struct AstroportCreatePairState {
    pub assets: Vec<Asset>,
    pub funds: Vec<Coin>,
}

#[cw_serde]
pub struct DojoswapCreatePairState {
    pub asset_infos: Vec<DojoswapAssetInfo>,
}

#[cw_serde]
pub enum ReplyHandler {
    AstroportCreatePair(AstroportCreatePairState),
    DojoswapCreatePair(DojoswapCreatePairState),
}

impl ReplyHandler {
    pub fn handle(
        self,
        deps: DepsMut,
        env: Env,
        reply: Reply,
    ) -> Result<Response, ContractError> {
        match self {
            Self::AstroportCreatePair(state) => on_astroport_create_pair_reply(deps, env, reply, state),
            Self::DojoswapCreatePair(state) => on_dojoswap_create_pair_reply(deps, env, reply, state),
        }
    }
}
