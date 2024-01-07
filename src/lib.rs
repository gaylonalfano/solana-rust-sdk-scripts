// Jeremy tip: https://youtu.be/PHbCmIckV20?t=262
pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

pub fn generate_keypair_from_json_file(path: &str) -> Result<solana_sdk::signer::keypair::Keypair> {
    // Q: How to create Keypair from JSON file using Rust?
    // REF: https://solana.stackexchange.com/questions/2067/how-can-i-read-a-keypair-from-a-json-file-that-contains-many-objects
    let file = std::fs::File::open(path).unwrap();
    let data: serde_json::Value = serde_json::from_reader(file).unwrap();
    let bytes: Vec<u8> = serde_json::from_value(data.clone()).unwrap();
    let signer_keypair = solana_sdk::signer::keypair::Keypair::from_bytes(&bytes).unwrap();

    Ok(signer_keypair)
}
