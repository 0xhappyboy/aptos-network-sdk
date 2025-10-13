use ring::signature::Ed25519KeyPair;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

use ring::signature::KeyPair;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    keypair: Vec<u8>,
}

impl Wallet {
    /// create new wallet
    pub fn new() -> Result<Self, String> {
        let rng = ring::rand::SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)
            .map_err(|e| format!("Failed to generate keypair: {}", e))?;

        Ok(Wallet {
            keypair: pkcs8_bytes.as_ref().to_vec(),
        })
    }

    pub fn from_pkcs8_bytes(pkcs8_bytes: &[u8]) -> Result<Self, String> {
        let _ = Ed25519KeyPair::from_pkcs8(pkcs8_bytes)
            .map_err(|e| format!("Invalid PKCS8 format: {}", e))?;

        Ok(Wallet {
            keypair: pkcs8_bytes.to_vec(),
        })
    }

    /// create wallet from private key
    pub fn from_private_key_hex(private_key_hex: &str) -> Result<Self, String> {
        let pkcs8_bytes =
            hex::decode(private_key_hex).map_err(|e| format!("Invalid hex: {}", e))?;
        Self::from_pkcs8_bytes(&pkcs8_bytes)
    }

    /// get keypair
    fn keypair(&self) -> Result<Ed25519KeyPair, String> {
        Ed25519KeyPair::from_pkcs8(&self.keypair)
            .map_err(|e| format!("Failed to load keypair: {}", e))
    }

    /// get wallet from base private key
    pub fn from_private_key_base64(private_key_base64: &str) -> Result<Self, String> {
        use base64::Engine as _;
        let pkcs8_bytes = base64::engine::general_purpose::STANDARD
            .decode(private_key_base64)
            .map_err(|e| format!("Invalid base64: {}", e))?;
        Self::from_pkcs8_bytes(&pkcs8_bytes)
    }

    /// get public key bytes
    pub fn public_key_bytes(&self) -> Result<Vec<u8>, String> {
        let keypair = self.keypair()?;
        Ok(keypair.public_key().as_ref().to_vec())
    }

    /// get public key hex
    pub fn public_key_hex(&self) -> Result<String, String> {
        let public_key = self.public_key_bytes()?;
        Ok(hex::encode(public_key))
    }

    /// get public key address
    pub fn address(&self) -> Result<String, String> {
        let public_key = self.public_key_bytes()?;
        let mut hasher = Sha3_256::new();
        hasher.update(&public_key);
        hasher.update(&[0u8]);
        let result = hasher.finalize();
        Ok(format!("0x{}", hex::encode(result)))
    }

    /// sign
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, String> {
        let keypair = self.keypair()?;
        let signature = keypair.sign(message);
        Ok(signature.as_ref().to_vec())
    }

    /// verify message
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<bool, String> {
        let public_key = self.public_key_bytes()?;
        let peer_public_key =
            ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, &public_key);
        match peer_public_key.verify(message, signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// get private key hex
    pub fn private_key_hex(&self) -> String {
        hex::encode(&self.keypair)
    }

    /// get private key base4
    pub fn private_key_base64(&self) -> String {
        use base64::Engine as _;
        base64::engine::general_purpose::STANDARD.encode(&self.keypair)
    }

    /// export keypair
    pub fn export_keypair(&self) -> Vec<u8> {
        self.keypair.clone()
    }

    /// clear wallet
    pub fn clear(mut self) {
        for byte in self.keypair.iter_mut() {
            *byte = 0;
        }
    }
}
