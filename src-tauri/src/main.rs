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

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn get_address() -> String {
//fn get_address(_wallet_name: &str) -> String {
    // generate entropy
    let words = 24;
    let entropy_bits = words * 11 * 32 / 33;
    let entropy_bytes = entropy_bits / 8;
    let entropy = vec![0u8; entropy_bytes];
    // TODO fill entropy with random bytes

    // convert entropy to mnemonic
    let m = Mnemonic::from_entropy(&entropy).unwrap();

    // convert mnemonic to seed
    let seed = m.to_seed("");

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
        .invoke_handler(tauri::generate_handler![get_address])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn secret_key_to_public_key(sk: FieldScalar) -> GE1 {
    ECPoint::generator_mul(&sk)
}
