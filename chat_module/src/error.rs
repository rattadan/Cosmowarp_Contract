use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Group already exists: {group_id}")]
    GroupAlreadyExists { group_id: String },

    #[error("Group not found: {group_id}")]
    GroupNotFound { group_id: String },

    #[error("Message not found: {message_id}")]
    MessageNotFound { message_id: u64 },

    #[error("Cannot modify message from another user")]
    CannotModifyOthersMessage {},

    #[error("Cannot delete message (not admin or owner)")]
    CannotDeleteMessage {},

    #[error("Invalid group ID format")]
    InvalidGroupId {},

    #[error("Group is not public")]
    GroupNotPublic {},
}
