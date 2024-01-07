use solana_sdk::signer::{keypair::Keypair, Signer};
fn main() {
    let kp = Keypair::new();

    println!("Pubkey:\n{}\n", &kp.pubkey().to_string());
    println!("Base58 private key:\n{}\n", &kp.to_base58_string());
    println!("JSON private key:\n{:?}", &kp.to_bytes());
}
