use cosmwasm_std::{Addr, DepsMut, Env, Order, Response, Uint128};
use schemars::JsonSchema;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::SerializeStruct;

use cw20::AllowanceResponse;
use cw_storage_plus::{Item, Map};

use crate::ContractError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin_address: Addr,
    pub minting_contract_address: Addr,
    pub platform_fees_collector_wallet: Addr,
    pub astro_proxy_address: Addr,
    pub platform_fee: Uint128,
    pub transaction_fee: Uint128,
    pub game_id: String,
}

pub const CONFIG_KEY: &str = "config";
pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

/// This is used for saving various vesting details
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub struct GameDetails {
    /// The game id
    pub game_id: String,

    /// Current status of the game - open, close, canceled
    pub game_status: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub struct FeeDetails {
    pub platform_fee: Uint128,
    pub transaction_fee: Uint128,
}


/// This is used for saving various vesting details
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub struct WalletPercentage {
    pub wallet_address: String,
    pub wallet_name: String,
    pub percentage: u32,
}


/// This is used for saving various vesting details
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub struct PoolTypeDetails {
    /// The pool type 
    pub pool_type: String,

    /// The min number of teams that must be present before the closing
    /// time else the pool gets dissolved
    pub min_teams_for_pool: u32,

    /// The max number of teams that can be accepted in the pool
    pub max_teams_for_pool: u32,

    /// The max number of teams allowed per gamer
    /// if head to head, then = 1
    pub max_teams_for_gamer: u32,

    /// The fee in tokens to enter the pool
    pub pool_fee: Uint128,

    /// Rake distribution 
    pub rake_list: Vec<WalletPercentage>,
}

/// This is used for saving various vesting details
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub struct PoolDetails {
    /// The pool id
    pub pool_id: String,

    /// The game id
    pub game_id: String,

    /// The pool type
    pub pool_type: String,

    /// How many teams are currently in the pool
    pub current_teams_count: u32,

    /// Whether rewards are distributed for this pool
    pub rewards_distributed: bool,
}


/// This is used for saving various vesting details
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub struct PoolTeamDetails {
    /// The pool id
    pub pool_id: String,

    /// The game id
    pub game_id: String,

    /// The pool type
    pub pool_type: String,

    /// The gamer address
    pub gamer_address: String,

    /// the team selected by the player
    pub team_id: String,

    /// reward amount in quantity of tokens after completion of game
    pub reward_amount: Uint128,

    /// whether the reward has been claimed
    pub claimed_reward: bool,

    /// refund amount in quantity of tokens in case game gets cancelled or pool not filled
    pub refund_amount: Uint128,

    /// whether the refund has been claimed
    pub claimed_refund: bool,

    /// team points updated after each game
    pub team_points: u64,

    /// team rank in the pool updated after each game
    pub team_rank: u64,
}

/// This is used for saving game result details
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub struct GameResult {
    pub gamer_address: String,
    pub game_id: String,
    pub team_id: String,
    pub reward_amount: Uint128,
    // UST
    pub refund_amount: Uint128,
    //  UST
    pub team_rank: u64,
    pub team_points: u64,
}

/// This is used for transferring tokens to multiple wallets
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub struct WalletTransferDetails {
    pub wallet_address: String,
    pub amount: Uint128,
}

pub const ALLOWANCES: Map<(&Addr, &Addr), AllowanceResponse> = Map::new("allowance");

/// Map of games. The key is game id and the
/// PoolDetails will contain information about the game
pub const GAME_DETAILS: Map<String, GameDetails> =
    Map::new("game_details");

/// Map of pools types. The key is pool type and the
/// PoolTypeDetails will contain information about the pool type
pub const POOL_TYPE_DETAILS: Map<String, PoolTypeDetails> =
    Map::new("pool_type_details");

/// Map of pools. The key is pool id and the
/// PoolDetails will contain information about the pool 
pub const POOL_DETAILS: Map<String, PoolDetails> =
    Map::new("pool_details");

// /// Map of pools and its gamers. the key is pool id and the
// /// PoolBettingDetails will contain information about the betters and amount betted
// pub const POOL_TEAM_DETAILS: Map<String, Vec<PoolTeamDetails>> =
//     Map::new("pool_team_details");
//


// The idea here to create a struct that can hold the map and then we implement secondary methods
// over this struct to get behaviour and properties of a Vector, but avoide the inremental gas fee
#[derive(Clone, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct PoolTeamDetailsMap<'a> {
    pub map: Map<'a, String, PoolTeamDetails>,

}

impl<T: de::DeserializeOwned> de::DeserializeOwned for PoolTeamDetailsMap<'_> {}

impl PoolTeamDetailsMap<'_> {
    fn turn_to_vector(&self, deps: DepsMut) -> Result<Vec<PoolTeamDetails>, ContractError> {
        let mut vector_to_return = Vec::new();
        for item in self.map.keys(deps.storage, None, None, Order::Ascending).into_iter() {
            let current_teamdetail = self.map.load(deps.storage, String::from_utf8(item)?)?;
            vector_to_return.push(current_teamdetail);
        }
        return Ok(vector_to_return);
    }
    fn add_to_map(&self, deps: DepsMut, env: Env, details: PoolTeamDetails) -> Result<Response, ContractError> {
        let new_key = self.generate_key(&env)?;
        self.map.save(deps.storage, new_key, &details)?;
        return Ok(Response::new())
    }
    fn generate_key(&self, env: &Env) -> String {
        // Todo find a more random function since multiple items saved at once would lead to override
        return format!("{}_{}_{}", env.contract.address, env.block.time.seconds(), env.block.height);
    }
}

impl Default for PoolTeamDetailsMap<'_> {
    fn default() -> Self {
        Self::new("temp_key")
    }
}

impl<'a> PoolTeamDetailsMap<'a> {
    fn new(
        new_key: &'a str,
    ) -> Self {
        Self {
            map: Map::new(new_key),
        }
    }
}

impl Serialize for PoolTeamDetailsMap<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut state = serializer.serialize_struct("PoolTeamDetailsMap", 1)?;
        state.serialize_field("map", &self.map)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for PoolTeamDetailsMap<'de> {
    fn deserialize<D>(deserializer: D) -> Result<PoolTeamDetailsMap<'de>, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("PoolTeamDetailsMap")
    }
}

/// Map of pools and its gamers. the key is pool id and the
/// PoolBettingDetails will contain information about the betters and amount betted
pub const POOL_TEAM_DETAILS: Map<String, PoolTeamDetailsMap> = Map::new("pool_team_details");


pub const CONTRACT_POOL_COUNT: Map<&Addr, Uint128> = Map::new("contract_pool_count");

pub const GAME_RESULT_DUMMY: Map<&Addr, GameResult> = Map::new("game_result");

pub const PLATFORM_WALLET_PERCENTAGES: Map<String, WalletPercentage> = Map::new("platform_wallet_percentages");

