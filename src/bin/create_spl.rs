use std::env;

// FIXME: Can't seem to get envy to find the .env file correctly,
// but after doing "$ export ENV_KEY=VALUE" in terminal, finally
// got the transaction to work. Sigh.
// REF: https://github.com/ronanyeah/solana-rust-examples/blob/master/src/bin/create_spl.rs
use solana_client::rpc_client::RpcClient;
// use std::env;
// use solana_rust_sdk_scripts::generate_keypair_from_json_file;
use dotenv::dotenv;
use solana_sdk::{
    instruction::Instruction,
    program_pack::Pack,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};
use spl_token::state::Mint;

#[derive(Debug, serde::Deserialize)]
struct Config {
    rpc_url: String,
    signer_keypair_base58: String,
    mint_keypair_base58: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // NOTE: I tried envy crate with Config struct but couldn't
    // get the env vars to ever load so swapped to dotenv.
    dotenv().expect("Failed to load .env file");

    let rpc_url = dotenv::var("RPC_URL").expect("Url not found!");
    let signer_keypair_base58 = dotenv::var("SIGNER_KEYPAIR_BASE58").expect("Keypair not found!");
    let mint_keypair_base58 = dotenv::var("MINT_KEYPAIR_BASE58").expect("Keypair not found!");

    let signer_wallet = Keypair::from_base58_string(&signer_keypair_base58);
    let token_mint = Keypair::from_base58_string(&mint_keypair_base58);
    let client = RpcClient::new(rpc_url);

    let decimals = 9;

    let minimum_balance_for_rent_exemption =
        client.get_minimum_balance_for_rent_exemption(Mint::LEN)?;

    let create_account_ix: Instruction = solana_sdk::system_instruction::create_account(
        &signer_wallet.pubkey(),
        &token_mint.pubkey(),
        minimum_balance_for_rent_exemption,
        Mint::LEN as u64,
        &spl_token::ID,
    );

    let initialize_mint_ix: Instruction = spl_token::instruction::initialize_mint(
        &spl_token::ID,
        &token_mint.pubkey(),
        &signer_wallet.pubkey(),
        None,
        decimals,
    )?;

    let recent_blockhash = client.get_latest_blockhash()?;

    let tx: Transaction = Transaction::new_signed_with_payer(
        &[create_account_ix, initialize_mint_ix],
        Some(&signer_wallet.pubkey()),
        &[&token_mint, &signer_wallet],
        recent_blockhash,
    );

    client.send_and_confirm_transaction_with_spinner(&tx)?;

    println!(
        "SPL Token Mint account with {} decimals created successfully:\n{}",
        decimals,
        token_mint.pubkey().to_string()
    );

    // Q: How to create Keypair from JSON file using Rust?
    // REF: https://solana.stackexchange.com/questions/2067/how-can-i-read-a-keypair-from-a-json-file-that-contains-many-objects
    // let file = std::fs::File::open("keypairs/signer-keypair.json").unwrap();
    // let data: serde_json::Value = serde_json::from_reader(file).unwrap();
    // let bytes: Vec<u8> = serde_json::from_value(data.clone()).unwrap();
    // let signer_keypair = Keypair::from_bytes(&bytes).unwrap();

    // let file = std::fs::File::open("keypairs/token-keypair.json").unwrap();
    // let data: serde_json::Value = serde_json::from_reader(file).unwrap();
    // let bytes: Vec<u8> = serde_json::from_value(data.clone()).unwrap();
    // let mint_keypair = Keypair::from_bytes(&bytes).unwrap();

    // U: Created a helper fn in lib.rs
    // let signer_keypair = generate_keypair_from_json_file("keypairs/signer-keypair.json")?;
    // let mint_keypair = generate_keypair_from_json_file("keypairs/token-keypair.json")?;
    // println!("signer pk: {:?}", signer_keypair.pubkey().to_string());
    // println!("signer b58: {:?}", signer_keypair.to_base58_string());
    //
    // println!("mint pk: {:?}", mint_keypair.pubkey().to_string());
    // println!("mint b58: {:?}", mint_keypair.to_base58_string());

    Ok(())
}
