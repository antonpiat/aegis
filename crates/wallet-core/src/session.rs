use models::{
    normalize_network_id, require_network, AppSettings, ChainFamily, RuntimeConfig, WalletFile,
    DEFAULT_NETWORK_ID,
};
use std::path::Path;
use std::sync::{Arc, Mutex};
use storage::{AppConfigStore, DeviceSecretStore, FileWalletStore};
use taurvia_solana::{configure_jupiter_api_key, Keypair, Pubkey, Signer, SolanaRpc};

use crate::WalletError;
use zeroize::Zeroize;

pub(crate) struct FamilyKeyring {
    pub solana: Keypair,
    pub evm: taurvia_evm::EvmSigner,
    pub bitcoin: taurvia_bitcoin::BtcSigner,
    pub bitcoin_testnet: taurvia_bitcoin::BtcSigner,
}

impl FamilyKeyring {
    pub fn from_mnemonic(mnemonic: &str) -> Result<Self, WalletError> {
        let seed = taurvia_hd::seed_from_mnemonic(mnemonic).map_err(|_| WalletError::InvalidMnemonic)?;
        let solana = taurvia_solana::derive_keypair_from_seed(seed.as_slice())
            .map_err(|_| WalletError::InvalidMnemonic)?;
        let evm = taurvia_evm::derive_from_seed(seed.as_slice()).map_err(WalletError::Operation)?;
        let bitcoin = taurvia_bitcoin::derive_from_seed(seed.as_slice(), false)
            .map_err(WalletError::Operation)?;
        let bitcoin_testnet = taurvia_bitcoin::derive_from_seed(seed.as_slice(), true)
            .map_err(WalletError::Operation)?;
        Ok(Self {
            solana,
            evm,
            bitcoin,
            bitcoin_testnet,
        })
    }

    pub fn from_solana_and_mnemonic(solana: Keypair, mnemonic: &str) -> Result<Self, WalletError> {
        let seed = taurvia_hd::seed_from_mnemonic(mnemonic).map_err(|_| WalletError::InvalidMnemonic)?;
        let evm = taurvia_evm::derive_from_seed(seed.as_slice()).map_err(WalletError::Operation)?;
        let bitcoin = taurvia_bitcoin::derive_from_seed(seed.as_slice(), false)
            .map_err(WalletError::Operation)?;
        let bitcoin_testnet = taurvia_bitcoin::derive_from_seed(seed.as_slice(), true)
            .map_err(WalletError::Operation)?;
        Ok(Self {
            solana,
            evm,
            bitcoin,
            bitcoin_testnet,
        })
    }

    pub fn address(&self, family: ChainFamily, testnet: bool) -> String {
        match family {
            ChainFamily::Solana => self.solana.pubkey().to_string(),
            ChainFamily::Evm => self.evm.address.clone(),
            ChainFamily::Bitcoin => {
                if testnet {
                    self.bitcoin_testnet.address.clone()
                } else {
                    self.bitcoin.address.clone()
                }
            }
            ChainFamily::Sui => String::new(),
        }
    }

    pub fn btc(&self, testnet: bool) -> &taurvia_bitcoin::BtcSigner {
        if testnet {
            &self.bitcoin_testnet
        } else {
            &self.bitcoin
        }
    }
}

impl Drop for FamilyKeyring {
    fn drop(&mut self) {
        // EVM / Bitcoin secrets are `Zeroizing`. Best-effort wipe of the Solana key copy.
        let mut bytes = self.solana.to_bytes();
        bytes.zeroize();
    }
}

pub(crate) struct WalletSession {
    pub keyring: FamilyKeyring,
}

pub struct WalletService {
    pub(crate) storage: FileWalletStore,
    pub(crate) config_store: AppConfigStore,
    pub(crate) settings: Mutex<AppSettings>,
    pub(crate) session: Arc<Mutex<Option<WalletSession>>>,
    pub(crate) cached_wallet: Mutex<Option<WalletFile>>,
    pub(crate) rpc: Mutex<SolanaRpc>,
    pub(crate) evm_rpc_url: Mutex<String>,
    pub(crate) btc_esplora: Mutex<String>,
    pub(crate) device_secrets: DeviceSecretStore,
}

impl WalletService {
    /// Desktop/mobile convenience: filesystem wallet store + OS device-secret store.
    pub fn new(data_dir: impl AsRef<Path>, _legacy_rpc_url: Option<&str>) -> Self {
        Self::with_device_secrets(data_dir, _legacy_rpc_url, DeviceSecretStore::os())
    }

    /// Same as `new`, but injects a device-secret backend (use memory in tests).
    pub fn with_device_secrets(
        data_dir: impl AsRef<Path>,
        _legacy_rpc_url: Option<&str>,
        device_secrets: DeviceSecretStore,
    ) -> Self {
        let data_dir = data_dir.as_ref().to_path_buf();
        let config_store = AppConfigStore::new(&data_dir);
        let settings = config_store.load().unwrap_or_default();
        let mut runtime = RuntimeConfig::resolve(&settings);
        if let Some(url) = _legacy_rpc_url.filter(|u| !u.is_empty()) {
            if settings
                .rpc_url
                .as_ref()
                .map(|s| s.trim().is_empty())
                .unwrap_or(true)
                && std::env::var("TAURVIA_RPC_URL").is_err()
            {
                runtime.rpc_url = url.to_string();
            }
        }
        Self::from_parts(
            settings,
            runtime,
            config_store,
            FileWalletStore::new(&data_dir),
            device_secrets,
        )
    }

    pub fn from_parts(
        settings: AppSettings,
        runtime: RuntimeConfig,
        config_store: AppConfigStore,
        storage: FileWalletStore,
        device_secrets: DeviceSecretStore,
    ) -> Self {
        configure_jupiter_api_key(runtime.jupiter_api_key.clone());
        let (sol_rpc, evm_url, btc_url) = split_runtime(&settings, &runtime);
        Self {
            storage,
            config_store,
            settings: Mutex::new(settings),
            session: Arc::new(Mutex::new(None)),
            cached_wallet: Mutex::new(None),
            rpc: Mutex::new(SolanaRpc::new(Some(&sol_rpc))),
            evm_rpc_url: Mutex::new(evm_url),
            btc_esplora: Mutex::new(btc_url),
            device_secrets,
        }
    }

    pub fn get_settings(&self) -> AppSettings {
        self.settings.lock().unwrap().clone()
    }

    pub fn update_settings(&self, settings: AppSettings) -> Result<RuntimeConfig, WalletError> {
        let prev = self.settings.lock().unwrap().clone();
        let mut settings = settings;
        settings.network = normalize_network_id(&settings.network).to_string();
        if self.storage.exists() {
            settings.network = normalize_network_id(&self.wallet_network()).to_string();
        }
        self.config_store.save(&settings)?;
        *self.settings.lock().unwrap() = settings.clone();
        let runtime = RuntimeConfig::resolve(&settings);
        let connectivity_changed = prev.rpc_url != settings.rpc_url
            || prev.rpc_urls != settings.rpc_urls
            || prev.jupiter_api_key != settings.jupiter_api_key
            || prev.network != settings.network;
        if connectivity_changed {
            configure_jupiter_api_key(runtime.jupiter_api_key.clone());
            let (sol_rpc, evm_url, btc_url) = split_runtime(&settings, &runtime);
            *self.rpc.lock().unwrap() = SolanaRpc::new(Some(&sol_rpc));
            *self.evm_rpc_url.lock().unwrap() = evm_url;
            *self.btc_esplora.lock().unwrap() = btc_url;
        }
        Ok(runtime)
    }

    pub fn wallet_network(&self) -> String {
        if let Some(wallet) = self.cached_wallet.lock().unwrap().as_ref() {
            return normalize_network_id(&wallet.network).to_string();
        }
        self.storage
            .load()
            .map(|w| normalize_network_id(&w.network).to_string())
            .unwrap_or_else(|_| DEFAULT_NETWORK_ID.to_string())
    }

    pub fn change_network(&self, network: &str) -> Result<RuntimeConfig, WalletError> {
        let id = normalize_network_id(network);
        let desc = require_network(id);
        if !desc.enabled {
            return Err(WalletError::Operation(anyhow::anyhow!(
                "{} is not enabled yet",
                desc.name
            )));
        }

        let wallet = self
            .cached_wallet
            .lock()
            .unwrap()
            .clone()
            .or_else(|| self.storage.load().ok())
            .ok_or(WalletError::NotFound)?;

        let mut updated = wallet;
        updated.network = id.to_string();
        self.storage.save(&updated)?;
        *self.cached_wallet.lock().unwrap() = Some(updated);

        let mut settings = self.get_settings();
        settings.network = id.to_string();
        settings.rpc_url = None;
        settings.rpc_urls.remove(id);
        self.update_settings(settings)
    }

    pub(crate) fn rpc_handle(&self) -> SolanaRpc {
        self.rpc.lock().unwrap().clone()
    }

    pub(crate) fn active_descriptor(&self) -> &'static models::NetworkDescriptor {
        require_network(&self.wallet_network())
    }

    pub fn wallet_exists(&self) -> bool {
        self.storage.exists()
    }

    pub fn is_unlocked(&self) -> bool {
        self.session.lock().unwrap().is_some()
    }

    pub fn get_public_key(&self) -> Option<String> {
        let desc = self.active_descriptor();
        if let Some(session) = self.session.lock().unwrap().as_ref() {
            return Some(session.keyring.address(desc.family, desc.is_testnet));
        }
        let wallet = self.cached_wallet.lock().unwrap().clone().or_else(|| self.storage.load().ok())?;
        if let Some(addr) = wallet.addresses.get(desc.family) {
            return Some(addr.to_string());
        }
        if desc.family == ChainFamily::Solana {
            return Some(wallet.public_key);
        }
        None
    }

    pub fn lock(&self) {
        let mut session = self.session.lock().unwrap();
        *session = None;
        *self.cached_wallet.lock().unwrap() = None;
    }

    pub(crate) fn signing_keypair(&self) -> Result<Keypair, WalletError> {
        let session = self.session.lock().unwrap();
        let session = session.as_ref().ok_or(WalletError::Locked)?;
        Keypair::try_from(session.keyring.solana.to_bytes().as_slice())
            .map_err(|_| WalletError::Locked)
    }

    pub(crate) fn require_pubkey(&self) -> Result<Pubkey, WalletError> {
        let session = self.session.lock().unwrap();
        let session = session.as_ref().ok_or(WalletError::Locked)?;
        Ok(session.keyring.solana.pubkey())
    }

    pub(crate) fn with_session<T>(
        &self,
        f: impl FnOnce(&FamilyKeyring) -> T,
    ) -> Result<T, WalletError> {
        let session = self.session.lock().unwrap();
        let session = session.as_ref().ok_or(WalletError::Locked)?;
        Ok(f(&session.keyring))
    }

    pub fn list_networks(&self) -> Vec<models::NetworkInfo> {
        models::list_network_info(false)
    }
}

fn split_runtime(settings: &AppSettings, runtime: &RuntimeConfig) -> (String, String, String) {
    let desc = require_network(&settings.network);
    let sol = if desc.family == ChainFamily::Solana {
        runtime.rpc_url.clone()
    } else {
        settings
            .rpc_urls
            .get(models::NETWORK_SOLANA_MAINNET)
            .cloned()
            .or_else(|| settings.rpc_url.clone())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| models::MANAGED_DEFAULT_RPC_URL.to_string())
    };
    let evm = if desc.family == ChainFamily::Evm {
        runtime.rpc_url.clone()
    } else {
        require_network(models::NETWORK_ETHEREUM_MAINNET)
            .default_rpc
            .to_string()
    };
    let btc = if desc.family == ChainFamily::Bitcoin {
        runtime.rpc_url.clone()
    } else {
        require_network(models::NETWORK_BITCOIN_MAINNET)
            .default_rpc
            .to_string()
    };
    (sol, evm, btc)
}
