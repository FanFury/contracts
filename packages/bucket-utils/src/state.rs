use cosmwasm_std::{Addr, CosmosMsg, DepsMut, QuerierWrapper, Response, to_binary, Uint128, WasmMsg};
use schemars::JsonSchema;

use bucket_contract::ContractError;
use bucket_contract::msg::{ExecuteMsg, QueryMsg};
use bucket_contract::state::PoolTeamDetails;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub struct Bucket {
    pub bucket_address: Addr,
}

impl Bucket {
    pub fn add_to_bucket(&self, data: PoolTeamDetails) -> Result<Response, ContractError> {
        return Ok(Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: String::from(self.bucket_address.clone()),
            msg: to_binary(&ExecuteMsg::AddToBucket { data })?,
            funds: vec![],
        })));
    }
    pub fn remove_from_bucket(&self, data: PoolTeamDetails) -> Result<Response, ContractError> {
        return Ok(Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: String::from(self.bucket_address.clone()),
            msg: to_binary(&ExecuteMsg::RemoveData { data })?,
            funds: vec![],
        })));
    }
    pub fn load(&self, querier: &QuerierWrapper) -> Result<Vec<PoolTeamDetails>, ContractError> {
        let data: Vec<PoolTeamDetails> = querier.query_wasm_smart(self.bucket_address.clone(), &QueryMsg::GetVector {})?;
        return Ok(data)
    }
    pub fn check_if_exits(&self, querier: &QuerierWrapper, data: PoolTeamDetails) -> Result<bool, ContractError> {
        let data: bool = querier.query_wasm_smart(self.bucket_address.clone(), &QueryMsg::CheckIfExist { data })?;
        return Ok(data)
    }
}
