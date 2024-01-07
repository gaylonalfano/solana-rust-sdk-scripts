use dotenv::dotenv;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    transaction::{Transaction, VersionedTransaction},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = dotenv::var("RPC_URL").expect("Url not found!");
    let signer_keypair_base58 = dotenv::var("SIGNER_KEYPAIR_BASE58").expect("Keypair not found!");
    let mint_keypair_base58 = dotenv::var("MINT_KEYPAIR_BASE58").expect("Keypair not found!");
    let receiver_wallet_keypair = Keypair::new();
    let receiver_wallet_pubkey = receiver_wallet_keypair.pubkey();

    let signer_wallet_keypair = Keypair::from_base58_string(&signer_keypair_base58);
    let token_mint = Keypair::from_base58_string(&mint_keypair_base58);
    let client = RpcClient::new(rpc_url);

    let amount = 1000000000;

    let receiver_token_account_pubkey = spl_associated_token_account::get_associated_token_address(
        &receiver_wallet_pubkey,
        &token_mint.pubkey(),
    );

    let create_associated_token_account_ix =
        spl_associated_token_account::instruction::create_associated_token_account(
            &signer_wallet_keypair.pubkey(),
            &receiver_wallet_pubkey,
            &token_mint.pubkey(),
            &spl_token::ID,
        );

    let mint_to_ix: Instruction = spl_token::instruction::mint_to(
        &spl_token::ID,
        &token_mint.pubkey(),
        &receiver_token_account_pubkey,
        &signer_wallet_keypair.pubkey(),
        &[&signer_wallet_keypair.pubkey()],
        amount,
    )?;

    let recent_blockhash = client.get_latest_blockhash()?;

    let tx: Transaction = Transaction::new_signed_with_payer(
        &[create_associated_token_account_ix, mint_to_ix],
        Some(&signer_wallet_keypair.pubkey()),
        &[&signer_wallet_keypair],
        recent_blockhash,
    );

    client.send_and_confirm_transaction_with_spinner(&tx)?;

    println!("SPL Tokens minted successfully.");
    println!("Amount: {}", amount);
    println!("Receiver pubkey: {}", receiver_wallet_pubkey);
    println!(
        "Associated token account: {}",
        receiver_token_account_pubkey
    );
    // -- Output:
    // SPL Tokens minted successfully.
    // Amount: 1000000000
    // Receiver pubkey: 5zBDBzvxd7S3ySLBoCdHE7v4LytBQowXuwnbYXi28keM
    // Associated token account: C5otpJeMdKpq9hEFwNUkawJiaXiYv7ABRXMsRVywYKvA

    Ok(())
}
