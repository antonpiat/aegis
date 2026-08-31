use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SendPreview {
    pub from: String,
    pub to: String,
    pub token: String,
    pub amount: String,
    pub network_name: String,
    pub estimated_fee: f64,
    pub fee_symbol: String,
    /// True when the recipient's associated token account will be created in this transfer.
    pub creates_token_account: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SendResult {
    #[serde(alias = "signature")]
    pub txid: String,
    pub status: String,
}
