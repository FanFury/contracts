use cosmwasm_std::Uint128;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub operator: String,
}

pub const STATE: Item<State> = Item::new("state");
// This counter will auto generate new keys for the values in the map
// Since Map supports iteration we can retrieve the data as a Vector
// And manage the data using it using different execute methods
pub const KEY_COUNTER: Item<u64> = Item::new("key_counter");

// Data To Store In Map as Vector
/// This is used for saving various vesting details
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub struct PoolTeamDetails {
    pub pool_id: String,
    pub game_id: String,
    pub pool_type: String,
    pub gamer_address: String,
    pub team_id: String,
    pub reward_amount: Uint128,
    pub claimed_reward: bool,
    pub refund_amount: Uint128,
    pub claimed_refund: bool,
    pub team_points: u64,
    pub team_rank: u64,
}

// --------------------------------------------------------------------------------------
pub const BUCKET: Map<String, PoolTeamDetails> = Map::new("bucket_store");