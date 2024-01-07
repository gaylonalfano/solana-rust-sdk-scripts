// FIXME: Todo: Start with reading the steps outlined here:
// REF1: https://www.notion.so/Solana-Quick-Reference-c0704fee2afa4ee5827ded6937ef47df?pvs=4#4303b45473d04783abced5a39f5c8d81
// REF2: https://solana.stackexchange.com/questions/8240/how-do-i-get-the-new-spl-token-metadata-interface-to-work-with-spl-token-2022s
// REF3: https://github.com/solana-labs/solana-program-library/blob/master/token-metadata/example/src/processor.rs#L73
//
// NOTE:
// - Official Token22 examples/tests: https://github.com/solana-labs/solana-program-library/blob/master/token/program-2022/tests/assert_instruction_count.rs
// - Get mint Keypair from JSON file using signer::Keypair::read_from_file()
// - Double-check mint keypair. Testing between persistent or not.
// Q: How to --enable-metadata but via SDK instead of CLI?
// A: Use spl-token-metadata-interface::instruction::initialize()
use std::env;

use solana_client::rpc_client::RpcClient;
// use std::env;
// use solana_rust_sdk_scripts::generate_keypair_from_json_file;
use dotenv::dotenv;
use solana_sdk::{
    instruction::Instruction,
    program_pack::Pack,
    signer::{keypair::Keypair, EncodableKey, EncodableKeypair, Signer},
    transaction::Transaction,
    transaction_context::InstructionAccount,
};
use spl_token_2022::extension::{
    metadata_pointer::instruction::InitializeInstructionData, token_metadata,
};
use spl_token_2022::state::{Account, Mint};
use spl_token_metadata_interface::state::TokenMetadata;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // NOTE: I tried envy crate with Config struct but couldn't
    // get the env vars to ever load so swapped to dotenv.
    dotenv().expect("Failed to load .env file");

    let rpc_url = dotenv::var("RPC_URL")?;
    let client = RpcClient::new(rpc_url);
    let signer_keypair_base58 = dotenv::var("SIGNER_KEYPAIR_BASE58")?;
    let signer_wallet = Keypair::from_base58_string(&signer_keypair_base58);
    // let mint_keypair_base58 = dotenv::var("MINT_KEYPAIR_BASE58")?;
    // let token22_mint_keypair = Keypair::read_from_file("./keypairs/token22-keypair.json")?;
    let token22_mint_keypair = Keypair::new();
    let decimals = 2;

    // region:       -- 1. Define Metadata & Compute Mint account size and Lamports
    // NOTE: Create account with size = ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::MetadataPointer]),
    // but lamports = rent-exempt amount for (above result + token_metadata.tlv_size_of()).
    // REF: https://solana.stackexchange.com/a/8252
    // Also see my smd/meme-coins/tina.ts example
    // NOTE: The TokenMetadata struct has a OptionalNonZeroPubkey custom type we need to use
    // REF: https://github.com/solana-labs/solana-program-library/blob/e08f30b3ae056dcec3aeca83b48707dae50e1a31/token-metadata/example/src/processor.rs#L73
    let update_authority =
        spl_pod::optional_keys::OptionalNonZeroPubkey::try_from(Some(signer_wallet.pubkey()))?;
    let token22_metadata = spl_token_metadata_interface::state::TokenMetadata {
        update_authority,
        mint: token22_mint_keypair.pubkey(),
        name: String::from("Token22 Name"),
        symbol: String::from("Token22 Symbol"),
        uri: String::from("Token22 Uri"),
        additional_metadata: vec![(String::from("key1"), String::from("value1"))],
    };

    // Size of MetadataExtension 2 bytes for type, 2 bytes for length
    let metadata_extension_size = 4;
    // Size of Mint Account with extension
    let mint_account_with_extension_size =
        spl_token_2022::extension::ExtensionType::try_calculate_account_len::<Mint>(&[
            spl_token_2022::extension::ExtensionType::MetadataPointer,
        ])?;
    // Size of metadata
    let token_metadata_size = token22_metadata.tlv_size_of()?;
    // Minimum lamports required for Mint Account
    let lamports = client.get_minimum_balance_for_rent_exemption(
        mint_account_with_extension_size + metadata_extension_size + token_metadata_size,
    )?;
    // endregion:    -- 1. Define Metadata & Compute Mint account size and Lamports

    // region:       -- 2. Instruction to invoke System Program to create new account
    // NOTE: For TS, this is from @solana/web3.js SystemProgram.createAccount
    let create_account_ix: Instruction = solana_sdk::system_instruction::create_account(
        &signer_wallet.pubkey(),
        &token22_mint_keypair.pubkey(),
        lamports,
        mint_account_with_extension_size as u64,
        &spl_token_2022::ID,
    );
    // endregion:    -- 2. Instruction to invoke System Program to create new account

    // region:       -- 3. Instruction to initialize the MetadataPointer Extension
    // Q: Not sure if I use metadata_pointer::instruction::initialize() or what...
    // NOTE: For TS, this is from @solana/spl-token createInitializeMetadataPointerInstruction
    let initialize_metadata_pointer_ix =
        spl_token_2022::extension::metadata_pointer::instruction::initialize(
            &spl_token_2022::ID,
            &token22_mint_keypair.pubkey(),
            Some(signer_wallet.pubkey()),
            // Q: Do I point metadata_address to the token mint address?
            // U: Doesn't seem to work. "Error: account or token already in use..."
            // U: After doing this in JS, I did point to the 'mint' Pubkey...
            Some(token22_mint_keypair.pubkey()),
        )?;
    // endregion:    -- 3. Instruction to initialize the MetadataPointer Extension

    // region:       -- 4. Instruction to initialize Mint Account data
    // NOTE: For TS, this is from: @solana/spl-token createInitializeMintInstruction
    let initialize_mint_ix = spl_token_2022::instruction::initialize_mint(
        &spl_token_2022::ID,
        &token22_mint_keypair.pubkey(),
        &signer_wallet.pubkey(),
        None,
        decimals,
    )?;
    // endregion:    -- 4. Instruction to initialize Mint Account data

    // region:       -- 5. Instruction to initialize Metadata Account data
    // Q: How to init a mint & enable metadata? Using the CLI, we
    // can simply pass --enable-metadata
    // U: I believe the CLI --enable-metadata does something with MetadataPointer
    // REF: https://github.com/solana-labs/solana-program-library/blob/5ee5487506e9f701b7d1e8425cbc063017b26c4c/token/cli/src/command.rs#L300
    // After the create-token command, you're prompted to do the following to init metadata:
    // To initialize metadata inside the mint, please run
    // `spl-token initialize-metadata <TOKEN_MINT> <YOUR_TOKEN_NAME> <YOUR_TOKEN_SYMBOL> <YOUR_TOKEN_URI>`, and sign with the mint authority
    // A: Turns out we use spl_token_metadata_interface::state::TokenMetadata struct,
    // which impls a handy TokenMetadata.tlv_size_of() to compute metadata size, which
    // is needed to compute the rent-exempt lamports needed for System Account creation.
    // See Step 1 above for details.
    // --- Here's a sample TokenMetadata struct: https://crates.io/crates/spl-token-metadata-interface
    // type Pubkey = [u8; 32];
    // type OptionalNonZeroPubkey = Pubkey; // if all zeroes, interpreted as `None`
    //
    // pub struct TokenMetadata {
    //     /// The authority that can sign to update the metadata
    //     pub update_authority: OptionalNonZeroPubkey,
    //     /// The associated mint, used to counter spoofing to be sure that metadata
    //     /// belongs to a particular mint
    //     pub mint: Pubkey,
    //     /// The longer name of the token
    //     pub name: String,
    //     /// The shortened symbol for the token
    //     pub symbol: String,
    //     /// The URI pointing to richer metadata
    //     pub uri: String,
    //     /// Any additional metadata about the token as key-value pairs. The program
    //     /// must avoid storing the same key twice.
    //     pub additional_metadata: Vec<(String, String)>,
    // }
    //
    // let token22_metadata = TokenMetadata {
    //     update_authority: signer_wallet.pubkey().to_bytes(),
    //     mint: token22_mint_keypair.pubkey().to_bytes(),
    //     name: String::from("Token22 Name"),
    //     symbol: String::from("Token22 Symbol"),
    //     uri: String::from("Token22 Uri"),
    //     additional_metadata: vec![(String::from("key1"), String::from("value1"))],
    // };

    // NOTE: For TS, this is from: @solana/spl-token-metadata createInitializeInstruction
    // NOTE: For Rust, use spl_token_metadata_interface crate: https://docs.rs/spl-token-metadata-interface/0.2.0/spl_token_metadata_interface/instruction/enum.TokenMetadataInstruction.html#variant.Initialize
    let initialize_metadata_ix = spl_token_metadata_interface::instruction::initialize(
        &spl_token_2022::ID,
        &token22_mint_keypair.pubkey(),
        &signer_wallet.pubkey(),
        &token22_mint_keypair.pubkey(),
        &signer_wallet.pubkey(),
        token22_metadata.name,
        token22_metadata.symbol,
        token22_metadata.uri,
    );
    // endregion:    -- 5. Instruction to initialize Metadata Account data

    // region:       -- 6. Instruction to update metadata with custome fields
    let (key, value) = &token22_metadata.additional_metadata[0];
    let update_metadata_field_ix = spl_token_metadata_interface::instruction::update_field(
        &spl_token_2022::ID,
        &token22_mint_keypair.pubkey(),
        &signer_wallet.pubkey(),
        // Q: How to pass Field and String without moving? clone?
        // A: Destructure and then use to_string()
        spl_token_metadata_interface::state::Field::Key(key.to_string()),
        value.to_string(),
    );
    // endregion:    -- 6. Instruction to update metadata with custome fields

    let recent_blockhash = client.get_latest_blockhash()?;

    let tx: Transaction = Transaction::new_signed_with_payer(
        &[
            create_account_ix,
            initialize_metadata_pointer_ix,
            initialize_mint_ix,
            initialize_metadata_ix,
            update_metadata_field_ix,
        ],
        Some(&signer_wallet.pubkey()),
        &[&token22_mint_keypair, &signer_wallet],
        recent_blockhash,
    );

    client.send_and_confirm_transaction_with_spinner(&tx)?;

    println!(
        "SPL Token22 Mint account with Metadata with {} decimals created successfully:\n{}",
        decimals,
        token22_mint_keypair.pubkey()
    );

    Ok(())
}
