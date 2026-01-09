use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub struct SolanaService {
    client: RpcClient,
}

impl SolanaService {
    pub fn new(url: &str) -> Self {
        SolanaService {
            client: RpcClient::new(url.to_string()),
        }
    }

    pub fn get_account_balance(&self, pubkey_str: &str) -> Result<u64, Box<dyn std::error::Error>> {
        let pubkey = Pubkey::from_str(pubkey_str)?;
        let balance = self.client.get_balance(&pubkey)?;
        Ok(balance)
    }
}
