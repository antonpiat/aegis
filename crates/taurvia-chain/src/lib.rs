//! Shared chain helpers: recipient checks, HTTP pool, USD prices.
//! No chain SDKs. Signing stays in family crates.

mod price;
mod validate;

pub use price::{native_price_usd, token_prices_usd};
pub use validate::validate_recipient;

use reqwest::Client;
use std::sync::OnceLock;
use std::time::Duration;

pub fn http_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("taurvia-wallet/0.4")
            .build()
            .expect("failed to build HTTP client")
    })
}
