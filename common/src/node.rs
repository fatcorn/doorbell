use solana_clap_utils::keypair::signer_from_path_with_config;
use solana_program::pubkey::Pubkey;
use solana_sdk::message::legacy::is_builtin_key_or_sysvar;
use solana_sdk::signature::read_keypair_file;
use solana_sdk::signer::Signer;
use crate::config::Config;
use borsh::{BorshDeserialize,BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct NodeInfo {
    /// the node pubkey id
    pub pubkey: Pubkey,
    /// the node nick name
    pub nick_name: String,
    /// the node address
    pub address: String,
    // #[borsh_skip]
    // pub signer: Box<dyn Signer>,
}

impl NodeInfo {
    pub fn get_from_config(config: &Config) -> Self {
        let keypair = read_keypair_file(config.keypair_path.clone()).unwrap();

        Self {
            pubkey: keypair.pubkey(),
            nick_name: config.nick_name.clone(),
            address: config.address.clone(),
            // signer: Box::new(keypair)
        }
    }
}