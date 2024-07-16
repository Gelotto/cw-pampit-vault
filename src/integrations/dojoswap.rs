use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

pub const DOJOSWAP_PAIR_FACTORY_ADDR: &str = "inj1pc2vxcmnyzawnwkf03n2ggvt997avtuwagqngk";

#[cw_serde]
pub enum DojoswapAssetInfo {
    Token { contract_addr: String },
    NativeToken { denom: String },
}

#[cw_serde]
pub struct DojoswapAsset {
    pub info: DojoswapAssetInfo,
    pub amount: Uint128,
}

#[cw_serde]
pub enum DojoswapFactoryExecuteMsg {
    CreatePair { assets: [DojoswapAsset; 2] },
}

#[cw_serde]
pub enum DojoswapFactoryQueryMsg {
    Pair { asset_infos: [DojoswapAssetInfo; 2] },
}

#[cw_serde]
pub struct DojoswapPairInfo {
    pub asset_infos: [DojoswapAssetInfo; 2],
    pub contract_addr: String,
    pub liquidity_token: String,
    pub asset_decimals: [u8; 2],
}
