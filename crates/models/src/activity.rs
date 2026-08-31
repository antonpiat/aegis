use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ActivityItem {
    #[serde(alias = "signature")]
    pub txid: String,
    pub timestamp: Option<i64>,
    pub status: String,
    pub direction: String,
    pub amount: Option<f64>,
    pub amount_symbol: Option<String>,
    pub description: String,
}
