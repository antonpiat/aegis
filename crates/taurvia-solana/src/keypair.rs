use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use models::DEFAULT_DERIVATION_PATH;
use solana_derivation_path::DerivationPath;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::keypair::keypair_from_seed_and_derivation_path;

pub fn derive_keypair_from_seed(seed: &[u8]) -> Result<Keypair> {
    let path = DerivationPath::from_absolute_path_str(DEFAULT_DERIVATION_PATH)
        .map_err(|e| anyhow!("invalid derivation path: {e}"))?;
    keypair_from_seed_and_derivation_path(seed, Some(path))
        .map_err(|e| anyhow!("key derivation failed: {e}"))
}

pub fn keypair_to_base64(keypair: &Keypair) -> String {
    BASE64.encode(keypair.to_bytes())
}

pub fn keypair_from_base64(encoded: &str) -> Result<Keypair> {
    let bytes = BASE64
        .decode(encoded)
        .context("invalid private key encoding")?;
    Keypair::try_from(bytes.as_slice()).map_err(|e| anyhow!("invalid keypair: {e}"))
}

/// Phantom / CLI / JSON array / hex / base58 / base64 Solana secret.
pub fn keypair_from_secret_input(input: &str) -> Result<Keypair> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        anyhow::bail!("private key is required");
    }
    if let Ok(bytes) = serde_json::from_str::<Vec<u8>>(trimmed) {
        return keypair_from_bytes(&bytes);
    }
    if let Ok(kp) = keypair_from_base64(trimmed) {
        return Ok(kp);
    }
    let hex = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    if (hex.len() == 64 || hex.len() == 128) && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        let bytes = hex::decode(hex).context("invalid hex private key")?;
        return keypair_from_bytes(&bytes);
    }
    if let Ok(bytes) = bs58::decode(trimmed).into_vec() {
        if bytes.len() == 32 || bytes.len() == 64 {
            return keypair_from_bytes(&bytes);
        }
    }
    anyhow::bail!("unrecognized Solana private key (use base58, JSON byte array, hex, or base64)")
}

fn keypair_from_bytes(bytes: &[u8]) -> Result<Keypair> {
    if bytes.len() == 32 {
        return solana_sdk::signature::keypair_from_seed(bytes)
            .map_err(|e| anyhow!("invalid Solana seed: {e}"));
    }
    Keypair::try_from(bytes).map_err(|e| anyhow!("invalid Solana keypair: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signer::Signer;

    #[test]
    fn phantom_vector_abandon() {
        // Phantom / Solflare path m/44'/501'/0'/0' (SLIP-0010).
        let kp = derive_keypair_from_mnemonic(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        )
        .unwrap();
        assert_eq!(
            kp.pubkey().to_string(),
            "HAgk14JpMQLgt6rVgv7cBQFJWFto5Dqxi472uT3DKpqk"
        );
    }

    fn derive_keypair_from_mnemonic(mnemonic: &str) -> Result<Keypair> {
        let seed = taurvia_hd::seed_from_mnemonic(mnemonic)?;
        derive_keypair_from_seed(seed.as_slice())
    }

    fn generate_mnemonic() -> Result<String> {
        taurvia_hd::generate_mnemonic()
    }

    #[test]
    fn mnemonic_round_trip() {
        let phrase = generate_mnemonic().unwrap();
        let kp1 = derive_keypair_from_mnemonic(&phrase).unwrap();
        let kp2 = derive_keypair_from_mnemonic(&phrase).unwrap();
        assert_eq!(kp1.pubkey(), kp2.pubkey());
    }

    #[test]
    fn keypair_base64_round_trip() {
        let phrase = generate_mnemonic().unwrap();
        let kp = derive_keypair_from_mnemonic(&phrase).unwrap();
        let encoded = keypair_to_base64(&kp);
        let restored = keypair_from_base64(&encoded).unwrap();
        assert_eq!(kp.pubkey(), restored.pubkey());
    }
}
