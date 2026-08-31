use serde::{Deserialize, Serialize};
use specta::Type;

use crate::TokenBalance;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WalletSnapshot {
    pub exists: bool,
    pub unlocked: bool,
    /// Active network id (e.g. `solana-mainnet`, `ethereum-mainnet`).
    pub network: String,
    /// Active-chain receive address (public).
    pub public_key: Option<String>,
    pub native_balance: Option<f64>,
    pub native_symbol: String,
    pub native_price_usd: Option<f64>,
    pub native_value_usd: Option<f64>,
    pub total_portfolio_usd: Option<f64>,
    pub tokens: Option<Vec<TokenBalance>>,
}
