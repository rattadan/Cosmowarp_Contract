use cosmwasm_schema::cw_serde;
use cw_storage_plus::Map;

/// User profile data - all fields except cosmos_address are optional
#[cw_serde]
pub struct UserProfile {
    /// The Cosmos address (bech32) - this is the key
    pub cosmos_address: String,
    /// Optional: Link to profile picture on IPFS
    pub profile_picture_ipfs: Option<String>,
    /// Optional: User's display name
    pub name: Option<String>,
    /// Optional: Company or business name
    pub company_name: Option<String>,
    /// Optional: Street address
    pub address: Option<String>,
    /// Optional: City
    pub city: Option<String>,
    /// Optional: Phone number
    pub phone: Option<String>,
    /// Optional: Email address
    pub email: Option<String>,
    /// Optional: Website URL
    pub website: Option<String>,
    /// Optional: Tax ID / VAT number
    pub tax_id: Option<String>,
}

impl UserProfile {
    /// Create a minimal profile with just the address
    pub fn new(cosmos_address: String) -> Self {
        Self {
            cosmos_address,
            profile_picture_ipfs: None,
            name: None,
            company_name: None,
            address: None,
            city: None,
            phone: None,
            email: None,
            website: None,
            tax_id: None,
        }
    }
}

/// Map from cosmos address -> UserProfile
pub const PROFILES: Map<&str, UserProfile> = Map::new("profiles");
