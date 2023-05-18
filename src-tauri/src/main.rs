// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aes_gcm_siv::{
    aead::{Aead, KeyInit},
    Aes256GcmSiv, Nonce // Or `Aes128GcmSiv`
};
use argon2::{
    password_hash::{
        PasswordHash, PasswordHasher, SaltString
    },
    Argon2
};
use bip39::Mnemonic;
use bls_ckd::{derive_master_sk, derive_child_sk};
use curv::elliptic::curves::{
    ECPoint,
    bls12_381::{
       g1::GE1,
       scalar::FieldScalar
    }
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::{Read, Write}};

// The number of unused addresses to keep in the wallet.
// This allows the recipient to receive funds without needing to decrypt
// the entropy each time.
const ADDRESS_GAP: u64 = 20;

#[derive(Serialize, Deserialize)]
struct Wallet {
    // The plain language name for the wallet.
    // Used for the filename that stores the wallet data.
    name: String,
    // A list of addresses. EIP2333 cannot store an xpub-style root public key
    // so a list of BLS public key addresses is stored (which are used to
    // receive funds).
    // This is a list of compressed GE1 bytes.
    addresses: Vec<Vec<u8>>,
    // The entropy that forms the basis of the secret keys.
    // This entropy is converted into a BIP39 mnemonic for manual backup.
    // The BIP39 mnemonic is converted into a seed, which is then used to
    // generate an EIP2333 root key.
    // The EIP2333 root key is used to derive secret keys using path m/i.
    // The corresponding public keys for each derived secret key is used to
    // populate the addresses list.
    encrypted_entropy: Vec<u8>,
    // The checksum is a signature using the first address to sign the
    // serialized
    // name+addresses+encrypted_entropy
    // encoded using MessagePack.
    // This allows any tampering with the addresslist to be detected and
    // ensures funds are received to addresses that can be spent using the
    // entropy.
    // TODO implement the checksum
    checksum: Vec<u8>,
    // A way to check if the decryptor has been entered by the user correctly.
    // This uses the argon2 hash function.
    decryptor_hash: String,
}

fn get_entropy(name: &str, decryptor: &str) -> Vec<u8> {
    // TODO store and retrieve entropy with encryption
    // TODO think about where to store the wallet file, eg $APP
    // if this wallet file exists, return the entropy for it
    match File::open(name) {
        Ok(mut file) => {
            let mut wallet_content = Vec::new();
            match file.read_to_end(&mut wallet_content) {
                Ok(_) => {
                    // TODO remove unwrap in line below
                    let wallet: Wallet = rmp_serde::from_slice(&wallet_content).unwrap();
                    decrypt_entropy(wallet.encrypted_entropy, decryptor)
                },
                Err(_) => Vec::<u8>::new(), // TODO return an error
            }
        }
        Err(_) => {
            // TODO decide if creating a new wallet is the right idea here
            // or if it should show an error like 'wallet not found'
            create_wallet(name, decryptor);
            get_entropy(name, decryptor)
        }
    }
}

fn mnemonic_from_entropy(entropy: Vec<u8>) -> Mnemonic {
    // convert entropy to mnemonic
    Mnemonic::from_entropy(&entropy).unwrap()
}

#[tauri::command]
fn get_mnemonic(wallet_name: &str, decryptor: &str) -> String {
    let entropy = get_entropy(wallet_name, decryptor);
    mnemonic_from_entropy(entropy).to_string()
}

#[tauri::command]
fn get_address(wallet_name: &str) -> String {
    match File::open(wallet_name) {
        Ok(mut file) => {
            let mut wallet_content = Vec::new();
            match file.read_to_end(&mut wallet_content) {
                Ok(_) => {
                    // TODO remove unwrap in line below
                    let wallet: Wallet = rmp_serde::from_slice(&wallet_content).unwrap();
                    // TODO remove clone below
                    hex::encode(wallet.addresses[0].clone())
                },
                Err(_) => {
                    // TODO revise this error message
                    "No valid address found for this wallet".to_string()
                }
            }
        }
        Err(_) => {
            // TODO decide if creating a new wallet is the right idea here
            // or whether there should be an error like 'wallet not found'
            let default_password = "insecure_default_password";
            create_wallet(wallet_name, default_password);
            get_address(wallet_name)
        }
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_address,
            get_mnemonic,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn secret_key_to_public_key(sk: FieldScalar) -> GE1 {
    ECPoint::generator_mul(&sk)
}

fn encrypt_entropy(decrypted_entropy: Vec<u8>, decryptor: &str) -> Vec<u8> {
    // hash the decryptor
    let stretched = stretch_password(decryptor);
    // TODO remove unwrap below
    let hash = PasswordHash::new(&stretched).unwrap().hash.unwrap();
    // use the hash with aes-gcm-siv to encrypt the entropy
    let cipher = Aes256GcmSiv::new(hash.as_bytes().into());
    // Using the same nonce across multiple messages is ok with siv, but would
    // not be ok with standard aes-gcm.
    // https://crypto.stackexchange.com/q/102334
    let nonce = Nonce::from_slice(b"unique nonce");
    // TODO remove unwrap below
    let encrypted_entropy = cipher.encrypt(nonce, decrypted_entropy.as_slice()).unwrap();
    encrypted_entropy
}

fn decrypt_entropy(encrypted_entropy: Vec<u8>, decryptor: &str) -> Vec<u8> {
    // hash the decryptor
    let stretched = stretch_password(decryptor);
    // TODO remove unwrap below
    let hash = PasswordHash::new(&stretched).unwrap().hash.unwrap();
    // use the hash with aes-gcm-siv to decrypt the entropy
    let cipher = Aes256GcmSiv::new(hash.as_bytes().into());
    // Using the same nonce across multiple messages is ok with siv, but would
    // not be ok with standard aes-gcm.
    // https://crypto.stackexchange.com/q/102334
    let nonce = Nonce::from_slice(b"unique nonce");
    // TODO remove unwrap below
    let decrypted_entropy = cipher.decrypt(nonce, encrypted_entropy.as_slice()).unwrap();
    decrypted_entropy
}

fn stretch_password(password: &str) -> String {
    // TODO confirm if fixed salt is ok here
    // TODO remove unwrap below
    let salt = SaltString::from_b64("d2FsbGV0d2FsbGV0d2FsbGV0").unwrap();
    let argon2 = Argon2::default();
    // TODO remove unwrap below
    let hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();
    hash.to_string()
}

fn create_wallet(name: &str, decryptor: &str) {
    // if wallet file doesn't exist, generate entropy
    let words = 24;
    let entropy_bits = words * 11 * 32 / 33;
    let entropy_bytes = entropy_bits / 8;
    let mut entropy = vec![0u8; entropy_bytes];
    // fill entropy with random bytes
    // TODO Check that thread_rng is secure enough
    rand::thread_rng().fill_bytes(&mut entropy);
    // convert entropy to mnemonic
    // TODO remove clone
    let m = mnemonic_from_entropy(entropy.clone());
    // convert mnemonic to seed
    let bip39_password = "";
    let seed = m.to_seed(bip39_password);
    // convert seed to root key
    let root_secret = derive_master_sk(&seed).unwrap();
    // populate the addresslist with several addresses
    let mut addresses = Vec::new();
    for i in 0..ADDRESS_GAP {
        let path = format!("m/{}", i).to_string();
        // use root key to derive a child key for a given path
        // TODO remove clone here
        let secret_key = derive_child_sk(root_secret.clone(), &path);
        let public_key = secret_key_to_public_key(secret_key);
        let address: Vec<u8> = public_key.serialize_compressed().to_vec();
        addresses.push(address);
    }
    // encrypt the entropy before saving
    // TODO remove clone below
    let encrypted_entropy = encrypt_entropy(entropy.clone(), decryptor);
    // create a wallet from this info
    let wallet = Wallet {
        name: name.to_string(),
        addresses: addresses,
        encrypted_entropy: encrypted_entropy,
        checksum: Vec::<u8>::new(),
        decryptor_hash: "".to_string(),
    };
    // serialize the wallet
    // TODO remove the unwrap below
    let wallet_bytes = rmp_serde::to_vec(&wallet).unwrap();
    // save this wallet for later use
    // TODO remove these unwraps and return an error
    let mut file = File::create(name).unwrap();
    file.write_all(&wallet_bytes).unwrap();
}
