use cosmwasm_std::{Deps, Order, StdResult};
use cw20::{AllAllowancesResponse, AllowanceInfo};

use crate::state::{ALLOWANCES}; // Import state related to allowances (make sure to replace it accordingly)
use cw_storage_plus::Bound;

// Settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

pub fn query_all_allowances(
    deps: Deps,
    owner: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<AllAllowancesResponse> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    // Replace ALLOWANCES with the native bank token storage item (make sure to replace it accordingly)
    let allowances: StdResult<Vec<AllowanceInfo>> = BANK_TOKEN_ALLOWANCES
        .prefix(&owner_addr) // Replace with the correct storage item for bank token allowances
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (k, v) = item?;
            Ok(AllowanceInfo {
                spender: String::from_utf8(k)?,
                allowance: v.allowance,
                expires: v.expires,
            })
        })
        .collect();
    Ok(AllAllowancesResponse {
        allowances: allowances?,
    })
}
