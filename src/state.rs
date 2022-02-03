use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Map, Item};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OwnerData {
    pub owner: Addr,
}

pub const OWNER_INFO: Item<OwnerData> = Item::new("owner_info");
pub const PRICES: Map<String, Uint128> = Map::new("prices");
