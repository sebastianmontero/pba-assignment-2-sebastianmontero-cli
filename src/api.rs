use hex::FromHex;
use parity_scale_codec::Decode;
use parity_scale_codec::Encode;
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::traits::Extrinsic;

use crate::error::Error;
use crate::key_store::*;
use crate::rpc::*;
use crate::types::*;

const SUPPLY_KEY: &[u8] = b"SUPPLY_KEY";
const MIN_FEE_KEY: &[u8] = b"MIN_FEE_KEY";
const BALANCES_PREFIX_KEY: &[u8] = b"BAL_";

pub struct API {
    rpc: RPC,
    pub keystore: KeyStore,
}

impl API {
    pub fn new(rpc: RPC, keystore: KeyStore) -> API {
        API { rpc, keystore }
    }

    pub async fn get_balance(&self, account: &str) -> Result<u128, Error> {
        let balance_key = Self::get_user_balance_key(KeyStore::get_pub_key(account)?.0);
        let value = self.get_value(&balance_key).await?;
        Ok(value)
    }

    pub async fn get_min_fee(&self) -> Result<u32, Error> {
        let value = self.get_value(MIN_FEE_KEY).await?;
        Ok(value)
    }

    pub async fn get_supply(&self) -> Result<u128, Error> {
        let value = self.get_value(SUPPLY_KEY).await?;
        Ok(value)
    }

    pub async fn set_min_fee(&self, fee: u32, tx_fee: u32) -> Result<String, Error> {
        let result = self
            .sign_and_send(&Self::get_admin_seed()?, Call::SetMinFee(fee), tx_fee)
            .await?;
        Ok(result)
    }

    pub async fn mint(&self, account: &str, amount: u128) -> Result<String, Error> {
        let result = self
            .sign_and_send(
                &Self::get_admin_seed()?,
                Call::Mint(KeyStore::get_pub_key(account)?.0, amount),
                0,
            )
            .await?;
        Ok(result)
    }

    pub async fn transfer(&self, from_mnemonic: &str, to_address: &str, amount: u128, tx_fee: u32) -> Result<String, Error> {
        let from_key = self.keystore.add(from_mnemonic)?;
        let to_key = KeyStore::get_pub_key(to_address)?;
        let result = self
            .sign_and_send(
                from_mnemonic,
                Call::Transfer(from_key.0, to_key.0, amount),
                tx_fee,
            )
            .await?;
        Ok(result)
    }

    pub async fn sign_and_send(
        &self,
        seed: &str,
        call: Call,
        tx_fee: u32,
    ) -> Result<String, Error> {
        let key = self.keystore.add(seed)?;
        let ext_payload = ExtrinsicPayload::new(call, tx_fee);
        let signature = self.keystore.sign(&key, &ext_payload.encode())?;
        let signature = Signature {
            signature: signature.clone(),
            origin: key.0.to_vec(),
        };
        let ext = BasicExtrinsic::new(ext_payload, Some(signature)).unwrap();
        println!("Encoded extrinsic: {:?}", HexDisplay::from(&ext.encode()));
        let response = self.call_extrinsic(ext).await?;
        Ok(response)
    }

    async fn call_extrinsic(&self, ext: BasicExtrinsic) -> Result<String, Error> {
        let param = HexDisplay::from(&ext.encode()).to_string();
        let response = self.rpc.request("author_submitExtrinsic", &param).await?;
        if let Some(trx) = response {
            return Ok(trx);
        }
        Err(Error::new("No response"))
    }

    async fn get_value<T: Decode>(&self, key: &[u8]) -> Result<T, Error> {
        let hex_key = HexDisplay::from(&key);
        let response = self
            .rpc
            .request("state_getStorage", &hex_key.to_string())
            .await?;
        if let Some(mut encoded) = response {
            encoded = encoded[2..].to_owned();
            let decoded = <Vec<u8>>::from_hex(encoded).unwrap();
            let value = T::decode(&mut &decoded[..])?;
            return Ok(value);
        }

        Err(Error::new("No value found"))
    }

    fn get_admin_seed() -> Result<String, Error> {
        let admin_seed = std::env::var("ADMIN_SEED")?;
        Ok(admin_seed)
    }

    fn get_user_balance_key(user: [u8; 32]) -> Vec<u8> {
        [&BALANCES_PREFIX_KEY[..], &user[..]].concat()
    }
}

pub fn do_set_fee(fee: u32) -> Result<(), String> {
    println!("Setting fee to: {}", fee);
    Ok(())
}
