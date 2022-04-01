use cosmwasm_std::{Addr, Env, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

static CONFIG_KEY: &[u8] = b"config";
static CONFIG_PRISM: &[u8] = b"config_prism";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: String,
    pub denom :String
}

pub fn config(storage: &mut dyn Storage) -> Singleton<State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn save_prism_address(storage: &mut dyn Storage) -> Singleton<String> {
    singleton(storage, CONFIG_PRISM)
}

pub fn read_prism_address(storage: &dyn Storage) -> ReadonlySingleton<String> {
    singleton_read(storage, CONFIG_PRISM)
}