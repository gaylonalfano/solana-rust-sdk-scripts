use chrono::prelude::*;
use solana_client::{
    rpc_client::RpcClient, rpc_response::RpcConfirmedTransactionStatusWithSignature,
};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::{
    str::FromStr,
    time::{Duration, UNIX_EPOCH},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = dotenv::var("RPC_URL")?;
    let client = RpcClient::new(rpc_url);

    let lookup_account_address = dotenv::var("TOKEN_MINT_PUBKEY")?;
    let lookup_account_pubkey: Pubkey = lookup_account_address.parse()?;

    let datetime = get_account_creation_date(&client, &lookup_account_pubkey)?;

    let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

    println!("{} creation date:", lookup_account_pubkey);
    println!("UTC - {}", timestamp_str);
    // -- Successful output:
    // FZKzL4wWoWpmUb6U4ggKGzDAKZUaMQWekfBF28cSBZE4 creation date:
    // UTC - 2023-12-24 11:27:24

    Ok(())
}

fn get_account_creation_date(
    rpc: &RpcClient,
    addr: &Pubkey,
) -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
    fn fetch(
        rpc: &RpcClient,
        addr: &Pubkey,
        before: Option<Signature>,
    ) -> Result<RpcConfirmedTransactionStatusWithSignature, Box<dyn std::error::Error>> {
        let mut sigs = rpc.get_signatures_for_address_with_config(
            addr,
            solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config {
                before,
                ..Default::default()
            },
        )?;

        sigs.sort_by_key(|sig| sig.block_time);

        let earliest = sigs.first().ok_or("Empty signature list!")?;

        if sigs.len() < 1000 {
            Ok(earliest.clone())
        } else {
            let sig = Signature::from_str(&earliest.signature)?;
            fetch(rpc, addr, Some(sig))
        }
    }

    let status = fetch(rpc, addr, None)?;

    let d = UNIX_EPOCH
        + Duration::from_secs(status.block_time.ok_or("Missing block time!")?.try_into()?);

    Ok(DateTime::<Utc>::from(d))
}
