use models::{ActivityItem, ChainFamily, TokenBalance, TokenInfo, WalletSnapshot};
use std::collections::HashMap;
use std::time::Duration;
use taurvia_solana::{get_metadata, get_prices, lamports_to_sol, WRAPPED_SOL_MINT};

use crate::session::WalletService;
use crate::WalletError;

const MARKET_DATA_BUDGET: Duration = Duration::from_secs(4);

impl WalletService {
    pub async fn get_snapshot(&self) -> Result<WalletSnapshot, WalletError> {
        let exists = self.wallet_exists();
        let network = self.wallet_network();
        let desc = models::require_network(&network);
        let empty = |unlocked: bool, public_key: Option<String>| WalletSnapshot {
            exists,
            unlocked,
            network: network.clone(),
            public_key,
            native_balance: None,
            native_symbol: desc.native_symbol.to_string(),
            native_price_usd: None,
            native_value_usd: None,
            total_portfolio_usd: None,
            tokens: None,
        };

        if !exists {
            return Ok(empty(false, None));
        }

        let unlocked = self.is_unlocked();
        let public_key = self.get_public_key();

        if !unlocked {
            return Ok(empty(false, public_key));
        }

        match desc.family {
            ChainFamily::Solana => self.solana_snapshot().await,
            ChainFamily::Evm => self.evm_snapshot().await,
            ChainFamily::Bitcoin => self.bitcoin_snapshot().await,
            ChainFamily::Sui => Err(WalletError::Operation(anyhow::anyhow!(
                "Sui is not enabled yet"
            ))),
        }
    }

    async fn evm_snapshot(&self) -> Result<WalletSnapshot, WalletError> {
        let url = self.evm_rpc_url.lock().unwrap().clone();
        let desc = *self.active_descriptor();
        let rpc = taurvia_evm::EvmRpc::new(&url, desc);
        let address = self.with_session(|k| k.evm.address.clone())?;
        rpc.snapshot(&address).await.map_err(WalletError::Operation)
    }

    async fn bitcoin_snapshot(&self) -> Result<WalletSnapshot, WalletError> {
        let url = self.btc_esplora.lock().unwrap().clone();
        let desc = *self.active_descriptor();
        let rpc = taurvia_bitcoin::BtcRpc::new(&url, desc);
        let address = self.with_session(|k| k.btc(desc.is_testnet).address.clone())?;
        rpc.snapshot(&address).await.map_err(WalletError::Operation)
    }

    async fn solana_snapshot(&self) -> Result<WalletSnapshot, WalletError> {
        let network = self.wallet_network();
        let public_key = self.get_public_key();
        let pubkey = self.require_pubkey()?;
        let rpc = self.rpc_handle();
        let (lamports, mut tokens) = rpc
            .get_balances_parallel(&pubkey)
            .await
            .map_err(WalletError::Operation)?;

        apply_local_metadata(&mut tokens);
        let mut mints: Vec<String> = tokens.iter().map(|token| token.mint.clone()).collect();
        if !mints.iter().any(|mint| mint == WRAPPED_SOL_MINT) {
            mints.push(WRAPPED_SOL_MINT.to_string());
        }
        let enrichment = tokio::time::timeout(MARKET_DATA_BUDGET, async {
            tokio::join!(get_metadata(&mints), get_prices(&mints))
        })
        .await;

        let mut native_price_usd = None;
        if let Ok((metadata, prices)) = enrichment {
            let prices = prices.unwrap_or_default();
            native_price_usd = prices.get(WRAPPED_SOL_MINT).copied();
            apply_remote_enrichment(&mut tokens, metadata.unwrap_or_default(), prices);
        }

        let native_balance = lamports_to_sol(lamports);
        let native_value_usd = native_price_usd.map(|price| price * native_balance);
        let tokens_value: f64 = tokens.iter().filter_map(|token| token.value_usd).sum();
        let total_portfolio_usd = Some(native_value_usd.unwrap_or(0.0) + tokens_value);

        Ok(WalletSnapshot {
            exists: true,
            unlocked: true,
            network,
            public_key,
            native_balance: Some(native_balance),
            native_symbol: "SOL".into(),
            native_price_usd,
            native_value_usd,
            total_portfolio_usd,
            tokens: Some(tokens),
        })
    }

    pub async fn get_activity(&self, limit: usize) -> Result<Vec<ActivityItem>, WalletError> {
        let desc = self.active_descriptor();
        match desc.family {
            ChainFamily::Solana => {
                let pubkey = self.require_pubkey()?;
                self.rpc_handle()
                    .get_activity(&pubkey, limit)
                    .await
                    .map_err(WalletError::Operation)
            }
            ChainFamily::Evm => {
                let address = self.with_session(|k| k.evm.address.clone())?;
                taurvia_evm::activity(*desc, &address, limit)
                    .await
                    .map_err(WalletError::Operation)
            }
            ChainFamily::Bitcoin => {
                let url = self.btc_esplora.lock().unwrap().clone();
                let rpc = taurvia_bitcoin::BtcRpc::new(&url, *desc);
                let address = self.with_session(|k| k.btc(desc.is_testnet).address.clone())?;
                rpc.activity(&address, limit)
                    .await
                    .map_err(WalletError::Operation)
            }
            ChainFamily::Sui => Ok(Vec::new()),
        }
    }
}

fn apply_local_metadata(tokens: &mut [TokenBalance]) {
    for token in tokens.iter_mut() {
        if let Some(info) = taurvia_solana::resolve_mint_local(&token.mint) {
            token.symbol = info.symbol;
            token.name = info.name;
            if info.decimals > 0 {
                token.decimals = info.decimals;
                if let Ok(raw) = token.amount.parse::<u64>() {
                    token.ui_amount = raw as f64 / 10f64.powi(info.decimals as i32);
                }
            }
            token.logo_uri = info.logo_uri;
        }
    }
}

fn apply_remote_enrichment(
    tokens: &mut [TokenBalance],
    metadata: HashMap<String, TokenInfo>,
    prices: HashMap<String, f64>,
) {
    for token in tokens.iter_mut() {
        if let Some(info) = metadata.get(&token.mint) {
            if !info.symbol.contains("...") {
                token.symbol = info.symbol.clone();
                token.name = info.name.clone();
            }
            if info.decimals > 0 {
                token.decimals = info.decimals;
                if let Ok(raw) = token.amount.parse::<u64>() {
                    token.ui_amount = raw as f64 / 10f64.powi(info.decimals as i32);
                }
            }
            if info.logo_uri.is_some() {
                token.logo_uri = info.logo_uri.clone();
            }
        }
        if let Some(price) = prices.get(&token.mint).copied() {
            token.price_usd = Some(price);
            token.value_usd = Some(price * token.ui_amount);
        }
    }
}
