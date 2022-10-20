use std::fs::File;
use std::io;
use std::path::Path;
use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};

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
            path.extend(&[".config", "sniffer", "config", "config.yml"]);
            path.to_str().unwrap().to_string()
        })
    };
}

/// The sniffer-helper configuration.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Config {
    /// assistant ip
    pub assistant_ip: String,
}

impl Config {
    /// Load a configuration from file.
    ///
    /// # Errors
    ///
    /// This function may return typical file I/O errors.
    pub fn load(config_file: String) -> Result<Self, io::Error> {
        load_config_file(config_file)
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

#[cfg(test)]
mod tests {
    use crate::config::{Config, CONFIG_FILE};

    #[test]
    fn test_load_config() {
        let config = Config::load((*CONFIG_FILE).clone().unwrap());
        println!("config {:?}", config.unwrap())
    }
}