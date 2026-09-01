use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SwapQuote {
    pub input_mint: String,
    pub output_mint: String,
    pub input_symbol: String,
    pub output_symbol: String,
    pub in_amount: String,
    pub out_amount: String,
    pub in_amount_ui: f64,
    pub out_amount_ui: f64,
    pub price_impact_pct: Option<f64>,
    pub network_fee: f64,
    pub fee_symbol: String,
    pub slippage_bps: u16,
    #[serde(default)]
    pub route: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SwapResult {
    pub signature: String,
    pub status: String,
}
