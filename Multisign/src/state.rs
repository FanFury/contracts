use cosmwasm_std::{Addr, Env, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

static CONFIG_KEY: &[u8] = b"config";
static CONFIG_CW3: &[u8] = b"config_cw3";
static CONFIG_CW20: &[u8] = b"config_cw20";


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: String,
}

pub fn config(storage: &mut dyn Storage) -> Singleton<State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn save_cw3_address(storage: &mut dyn Storage) -> Singleton<String> {
    singleton(storage, CONFIG_CW3)
}

pub fn read_cw3_address(storage: &dyn Storage) -> ReadonlySingleton<String> {
    singleton_read(storage, CONFIG_CW3)
}

pub fn save_cw20_address(storage: &mut dyn Storage) -> Singleton<String> {
    singleton(storage, CONFIG_CW20)
}

pub fn read_cw20_address(storage: &dyn Storage) -> ReadonlySingleton<String> {
    singleton_read(storage, CONFIG_CW20)
}