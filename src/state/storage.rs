use cosmwasm_std::{Addr, Timestamp, Uint128, Uint64};
use cw_storage_plus::{Item, Map};

use crate::tokens::TokenAmount;

use super::models::{Config, ReplyHandler};

pub const CONFIG: Item<Config> = Item::new("config");
pub const MANAGER: Item<Addr> = Item::new("manager");
pub const CREATED_AT: Item<Timestamp> = Item::new("created_at");
pub const CREATED_BY: Item<Addr> = Item::new("created_by");
pub const INITIAL_PLAY: Item<String> = Item::new("initial_play");
pub const INITIAL_QUOTE_TOKEN_AMOUNT: Item<TokenAmount> = Item::new("initial_quote_token_amount");
pub const INITIAL_BASE_TOKEN_AMOUNT: Item<TokenAmount> = Item::new("initial_base_token_amount");
pub const INITIAL_VIRTIAL_LIQUIDITY: Item<Uint128> = Item::new("initial_virtual_liquidity");
pub const REPLY_ID_COUNTER: Item<Uint64> = Item::new("reply_id_counter");
pub const REPLY_HANDLERS: Map<u64, ReplyHandler> = Map::new("reply_handlers");

pub const ASTROPORT_PAIR_ADDR: Item<Addr> = Item::new("astroport_pair_addr");
pub const DOJOSWAP_PAIR_ADDR: Item<Addr> = Item::new("dojoswap_pair_addr");
