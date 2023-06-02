// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod transfers;

use aes_gcm_siv::{
    aead::{Aead, KeyInit},
    Aes256GcmSiv, Nonce,
};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, Salt},
    Argon2,
};
use bip39::Mnemonic;
use bls_ckd::{derive_child_sk, derive_master_sk};
use curv::elliptic::curves::{
    bls12_381::{g1::GE1, scalar::FieldScalar},
    ECPoint,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    fs::File,
    io::{Read, Write},
    path::Path,
};

// The number of unused addresses to keep in the wallet.
// This allows the recipient to receive funds without needing to decrypt
// the entropy each time.
const ADDRESS_GAP: u64 = 20;

const WALLET_DIR: &str = "wallets";

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

fn get_entropy(name: &str, decryptor: &str) -> Result<Vec<u8>, String> {
    let wallet = read_wallet(name)?;
    decrypt_entropy(wallet.encrypted_entropy, decryptor)
}

fn mnemonic_from_entropy(entropy: Vec<u8>) -> Result<Mnemonic, String> {
    // convert entropy to mnemonic
    match Mnemonic::from_entropy(&entropy) {
        Ok(m) => Ok(m),
        Err(_) => Err("Error converting entropy to mnemonic".to_string()),
    }
}

#[tauri::command]
fn get_wallet_list() -> Result<Vec<String>, String> {
    ensure_wallet_dir_exists();
    let paths = match fs::read_dir(WALLET_DIR) {
        Ok(p) => p,
        Err(_) => return Err("Error reading wallet directory".to_string()),
    };
    let mut wallet_names = Vec::<String>::new();
    for wallet_name in paths {
        #[allow(clippy::single_match)]
        match wallet_name {
            Ok(os_filename) => {
                match os_filename.file_name().into_string() {
                    Ok(filename) => {
                        // TODO check this is a valid wallet, ie it can be
                        // parsed etc
                        wallet_names.push(filename)
                    }
                    Err(_) => { /* ignore unreadable wallet names */ }
                }
            }
            Err(_) => { /* ignore error, probably should do something */ }
        }
    }
    Ok(wallet_names)
}

#[tauri::command]
fn get_mnemonic(wallet_name: &str, decryptor: &str) -> Result<String, String> {
    // TODO check decryptor is correct by comparing with wallet.decryptor_hash
    let entropy = get_entropy(wallet_name, decryptor)?;
    let mnemonic = mnemonic_from_entropy(entropy)?;
    Ok(mnemonic.to_string())
}

#[tauri::command]
fn get_address(wallet_name: &str) -> Result<String, String> {
    let wallet = read_wallet(wallet_name)?;
    Ok(hex::encode(&wallet.addresses[0]))
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_address,
            get_mnemonic,
            create_new_random_wallet,
            restore_wallet,
            get_wallet_list,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn secret_key_to_public_key(sk: FieldScalar) -> GE1 {
    ECPoint::generator_mul(&sk)
}

fn encrypt_entropy(decrypted_entropy: Vec<u8>, decryptor: &str) -> Result<Vec<u8>, String> {
    // use argon2 hash of the decryptor as a symmetric key
    // hash the decryptor
    let cipher = symmetric_key_from_password(decryptor)?;
    // Using the same nonce across multiple messages is ok with siv, but would
    // not be ok with standard aes-gcm.
    // https://crypto.stackexchange.com/q/102334
    let nonce = Nonce::from_slice(b"unique nonce");
    // encrypt the entropy
    match cipher.encrypt(nonce, decrypted_entropy.as_slice()) {
        Ok(decrypted_entropy) => Ok(decrypted_entropy),
        Err(_) => Err("Error encrypting entropy".to_string()),
    }
}

fn decrypt_entropy(encrypted_entropy: Vec<u8>, decryptor: &str) -> Result<Vec<u8>, String> {
    // use argon2 hash of the decryptor as a symmetric key
    // hash the decryptor
    let cipher = symmetric_key_from_password(decryptor)?;
    // Using the same nonce across multiple messages is ok with siv, but would
    // not be ok with standard aes-gcm.
    // https://crypto.stackexchange.com/q/102334
    let nonce = Nonce::from_slice(b"unique nonce");
    // decrypt the entropy
    match cipher.decrypt(nonce, encrypted_entropy.as_slice()) {
        Ok(decrypted_entropy) => Ok(decrypted_entropy),
        Err(_) => Err("Error decrypting entropy".to_string()),
    }
}

fn stretch_password(password: &str) -> Result<PasswordHash, String> {
    // TODO confirm if fixed salt is ok here
    let salt = match Salt::from_b64("d2FsbGV0d2FsbGV0d2FsbGV0") {
        Ok(salt) => salt,
        Err(_) => return Err("Error creating salt for decryption".to_string()),
    };
    let argon2 = Argon2::default();
    match argon2.hash_password(password.as_bytes(), salt) {
        Ok(hash) => Ok(hash),
        Err(_) => Err("Error hashing password".to_string()),
    }
}

fn symmetric_key_from_password(password: &str) -> Result<Aes256GcmSiv, String> {
    let argon = stretch_password(password)?;
    let hash: Vec<u8> = match argon.hash {
        Some(hash) => hash.as_bytes().into(),
        None => return Err("Hash error".to_string()),
    };
    // use the hash with aes-gcm-siv to create a symmetric cipher
    match Aes256GcmSiv::new_from_slice(&hash) {
        Ok(cipher) => Ok(cipher),
        Err(_) => Err("Error creating symmetric key".to_string()),
    }
}

#[tauri::command]
fn restore_wallet(name: &str, decryptor: &str, mnemonic: &str) -> Result<String, String> {
    // parse mnemonic
    // TODO remove unwrap
    let m = Mnemonic::parse(mnemonic).unwrap();
    // TODO use result
    create_wallet_from_mnemonic(m, name, decryptor);
    Ok("".to_string())
}

#[tauri::command]
fn create_new_random_wallet(name: &str, decryptor: &str) -> Result<String, String> {
    // create entropy for a certain number of bip39 words
    let words = 24;
    let entropy_bits = words * 11 * 32 / 33;
    let entropy_bytes = entropy_bits / 8;
    let mut entropy = vec![0u8; entropy_bytes];
    // fill entropy with random bytes
    // TODO Check that thread_rng is secure enough
    rand::thread_rng().fill_bytes(&mut entropy);
    // convert entropy to mnemonic
    // TODO remove clone and unwrap
    let m = mnemonic_from_entropy(entropy.clone()).unwrap();
    create_wallet_from_mnemonic(m.clone(), name, decryptor);
    Ok(m.to_string())
}

fn create_wallet_from_mnemonic(m: Mnemonic, name: &str, decryptor: &str) {
    // convert mnemonic to entropy
    let entropy = m.to_entropy();
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
    // TODO remove unwrap below
    let encrypted_entropy = encrypt_entropy(entropy, decryptor).unwrap();
    // TODO remove unwrap below
    let decryptor_hash = stretch_password(decryptor).unwrap().to_string();
    // create a wallet from this info
    let wallet = Wallet {
        name: name.to_string(),
        addresses,
        encrypted_entropy,
        checksum: Vec::<u8>::new(),
        decryptor_hash,
    };
    // serialize the wallet
    // TODO remove the unwrap below
    let wallet_bytes = rmp_serde::to_vec(&wallet).unwrap();
    // save this wallet for later use
    // TODO remove these unwraps and return an error
    let wallet_location = wallet_file_path(name);
    let mut file = File::create(wallet_location).unwrap();
    file.write_all(&wallet_bytes).unwrap();
}

fn wallet_file_path(name: &str) -> String {
    // TODO validate this for bad chars, or is this automatic from tauri?
    // see https://tauri.app/v1/api/js/fs/#security
    // TODO consider if file read/write should be in the javascript, since it
    // seems to have the most docs, and the rust file api within tauri is quite
    // minimal. I prefer to keep as much of the key logic in rust, and avoid
    // exposing the javascript/webview to any secrets as much as possible.
    // TODO remove unwraps below
    ensure_wallet_dir_exists();
    Path::new(WALLET_DIR)
        .join(name)
        .to_str()
        .unwrap()
        .to_string()
}

fn ensure_wallet_dir_exists() {
    if !Path::new(WALLET_DIR).exists() {
        fs::create_dir_all(WALLET_DIR).unwrap();
    }
}

fn read_wallet(name: &str) -> Result<Wallet, String> {
    let wallet_location = wallet_file_path(name);
    // open file
    let mut file = match File::open(wallet_location) {
        Ok(file) => file,
        Err(_) => return Err("Error opening wallet file".to_string()),
    };
    // read file
    let mut wallet_content = Vec::new();
    match file.read_to_end(&mut wallet_content) {
        Ok(_) => {}
        Err(_) => return Err("Error reading wallet file".to_string()),
    }
    // parse file
    match rmp_serde::from_slice(&wallet_content) {
        Ok(w) => Ok(w),
        Err(_) => Err("Error parsing wallet file".to_string()),
    }
}
