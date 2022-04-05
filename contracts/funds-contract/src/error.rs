use cosmwasm_std::StdError;
use thiserror::Error;

use crate::state::PoolTeamDetails;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid reply ID (ID: {id:?}")]
    InvalidReplyId { id: u64 },

    #[error("Delete Failed, Item not found in bucket")]
    DeleteFailed { item: PoolTeamDetails },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
