use cosmwasm_schema::{cw_serde, QueryResponses};
use crate::state::UserProfile;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    /// Register or update user profile
    /// Only the owner of the address can update their own profile
    UpdateProfile {
        profile_picture_ipfs: Option<String>,
        name: Option<String>,
        company_name: Option<String>,
        address: Option<String>,
        city: Option<String>,
        phone: Option<String>,
        email: Option<String>,
        website: Option<String>,
        tax_id: Option<String>,
    },
    /// Delete user's own profile
    DeleteProfile {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get profile by cosmos address
    #[returns(ProfileResponse)]
    GetProfile { address: String },
    
    /// Check if profile exists
    #[returns(bool)]
    ProfileExists { address: String },
}

#[cw_serde]
pub struct ProfileResponse {
    pub profile: Option<UserProfile>,
}
