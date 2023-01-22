use crate::error::Error;
use sp_application_crypto::CryptoTypePublicPair;
use sp_core::sr25519::Public;
use sp_core::testing::SR25519;
use sp_keystore::{testing, SyncCryptoStore};
pub struct KeyStore {
    keystore: testing::KeyStore,
}

impl KeyStore {
    pub fn new() -> KeyStore {
        KeyStore {
            keystore: testing::KeyStore::new(),
        }
    }

    pub fn add(&self, mnemonic: &str) -> Result<Public, Error> {
        let key = SyncCryptoStore::sr25519_generate_new(&self.keystore, SR25519, Some(&mnemonic))?;
        Ok(key)
    }

    pub fn sign(&self, key: &Public, payload: &[u8]) -> Result<Vec<u8>, Error> {
        let keypair = CryptoTypePublicPair(sp_core::sr25519::CRYPTO_ID, key.0.into());
        let signature = SyncCryptoStore::sign_with(&self.keystore, SR25519, &keypair, &payload)?
            .ok_or(Error::new("Failed to sign payload"))?;
        Ok(signature)
    }

    pub fn get_pub_key(address: &str) -> Result<Public, Error> {
      let key :Public = address.parse()?;
      Ok(key)
    }
}
