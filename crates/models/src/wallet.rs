use serde::{Deserialize, Serialize};
use specta::Type;

pub const WALLET_FILE_VERSION: u32 = 2;
pub const MIN_WALLET_FILE_VERSION: u32 = 1;
pub const DEFAULT_DERIVATION_PATH: &str = "m/44'/501'/0'/0'";

/// How the wallet ciphertext is keyed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "kebab-case")]
pub enum WalletProtection {
    /// Argon2id(password) only — portable with JSON + password.
    #[default]
    Password,
    /// Argon2id(password) + OS keychain device secret — not portable off-device.
    PasswordDevice,
}

impl WalletProtection {
    pub fn is_device_bound(self) -> bool {
        matches!(self, Self::PasswordDevice)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CryptoEnvelope {
    pub kdf: String,
    pub salt: String,
    pub cipher: String,
    pub nonce: String,
    pub ciphertext: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct EncryptedPayload {
    pub mnemonic: String,
    #[serde(rename = "private_key")]
    pub private_key: String,
    pub derivation_path: String,
}

/// Public addresses keyed by family. Secrets are never stored here.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct WalletAddresses {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solana: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evm: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bitcoin: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sui: Option<String>,
}

impl WalletAddresses {
    pub fn get(&self, family: crate::ChainFamily) -> Option<&str> {
        match family {
            crate::ChainFamily::Solana => self.solana.as_deref(),
            crate::ChainFamily::Evm => self.evm.as_deref(),
            crate::ChainFamily::Bitcoin => self.bitcoin.as_deref(),
            crate::ChainFamily::Sui => self.sui.as_deref(),
        }
    }

    pub fn set(&mut self, family: crate::ChainFamily, address: String) {
        match family {
            crate::ChainFamily::Solana => self.solana = Some(address),
            crate::ChainFamily::Evm => self.evm = Some(address),
            crate::ChainFamily::Bitcoin => self.bitcoin = Some(address),
            crate::ChainFamily::Sui => self.sui = Some(address),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WalletFile {
    pub version: u32,
    pub wallet_id: String,
    pub network: String,
    pub public_key: String,
    pub created_at: String,
    /// Absent in older files → password-only.
    #[serde(default)]
    pub protection: WalletProtection,
    /// Public family addresses. Missing on v1 files.
    #[serde(default)]
    pub addresses: WalletAddresses,
    pub crypto: CryptoEnvelope,
}
