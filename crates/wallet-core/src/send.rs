use models::{ChainFamily, SendPreview, SendResult};

use crate::session::WalletService;
use crate::WalletError;

impl WalletService {
    pub async fn preview_send(
        &self,
        to: &str,
        amount: f64,
        asset: Option<&str>,
    ) -> Result<SendPreview, WalletError> {
        let desc = self.active_descriptor();
        taurvia_chain::validate_recipient(desc.family, to).map_err(WalletError::Operation)?;
        match desc.family {
            ChainFamily::Solana => {
                let native = asset
                    .map(|a| a.eq_ignore_ascii_case("sol") || a.eq_ignore_ascii_case("native"))
                    .unwrap_or(true);
                if native {
                    self.preview_sol_send(to, amount).await
                } else {
                    self.preview_spl_send(asset.unwrap_or(""), to, amount).await
                }
            }
            ChainFamily::Evm => {
                let url = self.evm_rpc_url.lock().unwrap().clone();
                let rpc = taurvia_evm::EvmRpc::new(&url, *desc);
                let signer = self.with_session(|k| k.evm.clone())?;
                rpc.preview_send(&signer, to, amount, asset)
                    .await
                    .map_err(WalletError::Operation)
            }
            ChainFamily::Bitcoin => {
                let url = self.btc_esplora.lock().unwrap().clone();
                let rpc = taurvia_bitcoin::BtcRpc::new(&url, *desc);
                let signer = self.with_session(|k| k.btc(desc.is_testnet).clone())?;
                rpc.preview_send(&signer, to, amount)
                    .await
                    .map_err(WalletError::Operation)
            }
            ChainFamily::Sui => Err(WalletError::Operation(anyhow::anyhow!(
                "Sui is not enabled yet"
            ))),
        }
    }

    pub async fn send_transfer(
        &self,
        password: &str,
        to: &str,
        amount: f64,
        asset: Option<&str>,
    ) -> Result<SendResult, WalletError> {
        self.verify_password(password)?;
        let desc = self.active_descriptor();
        taurvia_chain::validate_recipient(desc.family, to).map_err(WalletError::Operation)?;
        match desc.family {
            ChainFamily::Solana => {
                let native = asset
                    .map(|a| a.eq_ignore_ascii_case("sol") || a.eq_ignore_ascii_case("native"))
                    .unwrap_or(true);
                if native {
                    self.send_sol_unlocked(to, amount).await
                } else {
                    self.send_spl_unlocked(asset.unwrap_or(""), to, amount).await
                }
            }
            ChainFamily::Evm => {
                let url = self.evm_rpc_url.lock().unwrap().clone();
                let rpc = taurvia_evm::EvmRpc::new(&url, *desc);
                let signer = self.with_session(|k| k.evm.clone())?;
                rpc.send(&signer, to, amount, asset)
                    .await
                    .map_err(WalletError::Operation)
            }
            ChainFamily::Bitcoin => {
                let url = self.btc_esplora.lock().unwrap().clone();
                let rpc = taurvia_bitcoin::BtcRpc::new(&url, *desc);
                let signer = self.with_session(|k| k.btc(desc.is_testnet).clone())?;
                rpc.send(&signer, to, amount)
                    .await
                    .map_err(WalletError::Operation)
            }
            ChainFamily::Sui => Err(WalletError::Operation(anyhow::anyhow!(
                "Sui is not enabled yet"
            ))),
        }
    }

    pub async fn preview_sol_send(
        &self,
        to: &str,
        amount_sol: f64,
    ) -> Result<SendPreview, WalletError> {
        taurvia_chain::validate_recipient(ChainFamily::Solana, to)
            .map_err(WalletError::Operation)?;
        let keypair = self.signing_keypair()?;
        let mut preview = self
            .rpc_handle()
            .preview_sol_send(&keypair, to, amount_sol)
            .await
            .map_err(WalletError::Operation)?;
        preview.network_name = self.active_descriptor().name.to_string();
        Ok(preview)
    }

    pub async fn preview_spl_send(
        &self,
        mint: &str,
        to: &str,
        amount: f64,
    ) -> Result<SendPreview, WalletError> {
        taurvia_chain::validate_recipient(ChainFamily::Solana, to)
            .map_err(WalletError::Operation)?;
        let keypair = self.signing_keypair()?;
        let mut preview = self
            .rpc_handle()
            .preview_spl_send(&keypair, mint, to, amount)
            .await
            .map_err(WalletError::Operation)?;
        preview.network_name = self.active_descriptor().name.to_string();
        Ok(preview)
    }

    pub async fn send_sol(
        &self,
        password: &str,
        to: &str,
        amount_sol: f64,
    ) -> Result<SendResult, WalletError> {
        self.verify_password(password)?;
        self.send_sol_unlocked(to, amount_sol).await
    }

    pub async fn send_spl(
        &self,
        password: &str,
        mint: &str,
        to: &str,
        amount: f64,
    ) -> Result<SendResult, WalletError> {
        self.verify_password(password)?;
        self.send_spl_unlocked(mint, to, amount).await
    }

    async fn send_sol_unlocked(&self, to: &str, amount_sol: f64) -> Result<SendResult, WalletError> {
        taurvia_chain::validate_recipient(ChainFamily::Solana, to)
            .map_err(WalletError::Operation)?;
        let keypair = self.signing_keypair()?;
        self.rpc_handle()
            .send_sol(&keypair, to, amount_sol)
            .await
            .map_err(WalletError::Operation)
    }

    async fn send_spl_unlocked(
        &self,
        mint: &str,
        to: &str,
        amount: f64,
    ) -> Result<SendResult, WalletError> {
        taurvia_chain::validate_recipient(ChainFamily::Solana, to)
            .map_err(WalletError::Operation)?;
        let keypair = self.signing_keypair()?;
        self.rpc_handle()
            .send_spl(&keypair, mint, to, amount)
            .await
            .map_err(WalletError::Operation)
    }
}
