use models::TokenBalance;

#[derive(Clone, Copy)]
pub struct CuratedToken {
    pub address: &'static str,
    pub symbol: &'static str,
    pub name: &'static str,
    pub decimals: u8,
}

/// Ethereum mainnet curated set. L2s can swap this list later via descriptor id.
pub fn curated_tokens(network_id: &str) -> &'static [CuratedToken] {
    match network_id {
        "ethereum-mainnet" => ETHEREUM_MAINNET,
        _ => &[],
    }
}

const ETHEREUM_MAINNET: &[CuratedToken] = &[
    CuratedToken {
        address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
        symbol: "WETH",
        name: "Wrapped Ether",
        decimals: 18,
    },
    CuratedToken {
        address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        symbol: "USDC",
        name: "USD Coin",
        decimals: 6,
    },
    CuratedToken {
        address: "0xdAC17F958D2ee523a2206206994597C13D831ec7",
        symbol: "USDT",
        name: "Tether USD",
        decimals: 6,
    },
    CuratedToken {
        address: "0x6B175474E89094C44Da98b954EedeAC495271d0F",
        symbol: "DAI",
        name: "Dai Stablecoin",
        decimals: 18,
    },
    CuratedToken {
        address: "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599",
        symbol: "WBTC",
        name: "Wrapped BTC",
        decimals: 8,
    },
];

pub fn token_balance(
    token: &CuratedToken,
    raw: alloy::primitives::U256,
    price_usd: Option<f64>,
) -> Option<TokenBalance> {
    if raw.is_zero() {
        return None;
    }
    let ui = u256_to_f64(raw, token.decimals);
    Some(TokenBalance {
        mint: token.address.to_string(),
        symbol: token.symbol.to_string(),
        name: token.name.to_string(),
        amount: raw.to_string(),
        decimals: token.decimals,
        ui_amount: ui,
        logo_uri: None,
        price_usd,
        value_usd: price_usd.map(|p| p * ui),
    })
}

pub fn u256_to_f64(value: alloy::primitives::U256, decimals: u8) -> f64 {
    let s = value.to_string();
    let raw: f64 = s.parse().unwrap_or(0.0);
    raw / 10f64.powi(decimals as i32)
}

pub fn f64_to_u256(amount: f64, decimals: u8) -> alloy::primitives::U256 {
    let scaled = (amount * 10f64.powi(decimals as i32)).round();
    alloy::primitives::U256::from(scaled as u128)
}
