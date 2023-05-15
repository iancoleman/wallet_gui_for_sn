// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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
use std::{fs::File, io::{Read, Write}};

fn get_entropy(name: &str) -> Vec<u8> {
    // TODO store and retrieve entropy with encryption
    // TODO think about where to store the wallet file, eg $APP
    // if this wallet file exists, return the entropy for it
    match File::open(name) {
        Ok(mut file) => {
            let mut entropy = Vec::new();
            match file.read_to_end(&mut entropy) {
                Ok(_) => entropy,
                Err(_) => entropy, // TODO return an error
            }
        }
        Err(_) => {
            // if wallet file doesn't exist, generate entropy
            let words = 24;
            let entropy_bits = words * 11 * 32 / 33;
            let entropy_bytes = entropy_bits / 8;
            let mut entropy = vec![0u8; entropy_bytes];
            // fill entropy with random bytes
            // TODO Check that thread_rng is secure enough
            rand::thread_rng().fill_bytes(&mut entropy);
            // save this entropy for later use
            // TODO remove these unwraps and return an error
            let mut file = File::create(name).unwrap();
            file.write_all(&entropy).unwrap();
            return entropy;
        }
    }
}

fn mnemonic_from_entropy(entropy: Vec<u8>) -> Mnemonic {
    // convert entropy to mnemonic
    Mnemonic::from_entropy(&entropy).unwrap()
}

#[tauri::command]
fn get_mnemonic(wallet_name: &str) -> String {
    let entropy = get_entropy(wallet_name);
    mnemonic_from_entropy(entropy).to_string()
}

#[tauri::command]
fn get_address(wallet_name: &str) -> String {

    let entropy = get_entropy(wallet_name);
    let m = mnemonic_from_entropy(entropy);

    // convert mnemonic to seed
    let password = "";
    let seed = m.to_seed(password);

    // convert seed to root key
    let root_secret = derive_master_sk(&seed).unwrap();

    // use root key to derive a child key for a given path
    let path = "m/0".to_string();
    let secret_key = derive_child_sk(root_secret, &path);
    let public_key = secret_key_to_public_key(secret_key);
    hex::encode(public_key.serialize_compressed())
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
