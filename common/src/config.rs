use std::fs::{create_dir_all, File};
use std::io::Write;
// Wallet settings that can be configured for long-term use
use {
    serde_derive::{Deserialize, Serialize},
    std::{io, path::Path},
    // url::Url,
    lazy_static::lazy_static
};

lazy_static! {
    /// The default path to the CLI configuration file.
    ///
    /// This is a [lazy_static] of `Option<String>`, the value of which is
    ///
    /// > `~/.config/solana/cli/config.yml`
    ///
    /// It will only be `None` if it is unable to identify the user's home
    /// directory, which should not happen under typical OS environments.
    ///
    /// [lazy_static]: https://docs.rs/lazy_static
    pub static ref CONFIG_FILE: Option<String> = {
        dirs_next::home_dir().map(|mut path| {
            path.extend(&[".config", "doorbell", "config", "config.yml"]);
            path.to_str().unwrap().to_string()
        })
    };
}

/// The Doorbell CLI configuration.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Config {
    /// The signing source can be loaded with either the `signer_from_path`
    /// function, or with `solana_clap_utils::keypair::DefaultSigner`.
    pub keypair_path: String,
    /// the data base path
    pub data_path: String,
    /// the node nick name
    pub nick_name: String,
    /// the node open address for connect
    pub address: String,

}

impl Default for Config {
    fn default() -> Self {
        let keypair_path = {
            let mut keypair_path = dirs_next::home_dir().expect("home directory");
            keypair_path.extend(&[".config", "doorbell", "id.json"]);
            keypair_path.to_str().unwrap().to_string()
        };
        let data_path = {
            let mut data_path = dirs_next::home_dir().expect("home directory");
            data_path.extend(&[".config", "doorbell", "data"]);
            data_path.to_str().unwrap().to_string()
        };

        let nick_name = "no_123456".to_string();
        let address = "127.0.0.1:9898".to_string();
        Self {
            keypair_path,
            data_path,
            nick_name,
            address
        }
    }
}

impl Config {
    /// Load a configuration from file.
    ///
    /// # Errors
    ///
    /// This function may return typical file I/O errors.
    pub fn load(config_file: &str) -> Result<Self, io::Error> {
        load_config_file(config_file)
    }

    /// Save a configuration to file.
    ///
    /// If the file's directory does not exist, it will be created. If the file
    /// already exists, it will be overwritten.
    ///
    /// # Errors
    ///
    /// This function may return typical file I/O errors.
    pub fn save(&self, config_file: &str) -> Result<(), io::Error> {
        save_config_file(self, config_file)
    }
}

/// Load a value from a file in YAML format.
///
/// Despite the name, this function is generic YAML file deserializer, a thin
/// wrapper around serde.
///
/// Most callers should instead use [`Config::load`].
///
/// # Errors
///
/// This function may return typical file I/O errors.
pub fn load_config_file<T, P>(config_file: P) -> Result<T, io::Error>
    where
        T: serde::de::DeserializeOwned,
        P: AsRef<Path>,
{
    let file = File::open(config_file)?;
    let config = serde_yaml::from_reader(file)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{:?}", err)))?;
    Ok(config)
}

/// Save a value to a file in YAML format.
///
/// Despite the name, this function is a generic YAML file serializer, a thin
/// wrapper around serde.
///
/// If the file's directory does not exist, it will be created. If the file
/// already exists, it will be overwritten.
///
/// Most callers should instead use [`Config::save`].
///
/// # Errors
///
/// This function may return typical file I/O errors.
pub fn save_config_file<T, P>(config: &T, config_file: P) -> Result<(), io::Error>
    where
        T: serde::ser::Serialize,
        P: AsRef<Path>,
{
    let serialized = serde_yaml::to_string(config)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{:?}", err)))?;

    if let Some(outdir) = config_file.as_ref().parent() {
        create_dir_all(outdir)?;
    }
    let mut file = File::create(config_file)?;
    file.write_all(&serialized.into_bytes())?;

    Ok(())
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn compute_websocket_url() {
        assert_eq!(
            Config::compute_websocket_url("http://api.devnet.solana.com"),
            "ws://api.devnet.solana.com/".to_string()
        );

        assert_eq!(
            Config::compute_websocket_url("https://api.devnet.solana.com"),
            "wss://api.devnet.solana.com/".to_string()
        );

        assert_eq!(
            Config::compute_websocket_url("http://example.com:8899"),
            "ws://example.com:8900/".to_string()
        );
        assert_eq!(
            Config::compute_websocket_url("https://example.com:1234"),
            "wss://example.com:1235/".to_string()
        );

        assert_eq!(Config::compute_websocket_url("garbage"), String::new());
    }
}
