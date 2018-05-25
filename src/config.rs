use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::net::SocketAddr;
use std::path::Path;

use toml;

const DEFAULT_CONFIG: &'static str = {
r#"secret = "this is a secret secret you should not know about"
bind = "0.0.0.0:8000"

[commands]
push = 'cat /dev/stdin && echo ""'
"#
};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub commands: HashMap<String, String>,
    pub secret: String,
    pub bind: SocketAddr,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Config {
        if !path.as_ref().exists() {
            File::create(&path).expect("Failed to create a new config file")
                .write_all(DEFAULT_CONFIG.as_bytes())
                .expect("Failed to write to a new config file");
        }

        toml::from_str(
            &fs::read_to_string(path)
                .expect("Failed to read the config file"))
            .expect("Failed to parse the config file as a TOML file")
    }
}
