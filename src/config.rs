use std::{fs::File, io::Read, net::SocketAddr, path::{Path, PathBuf}, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    bind_addr: SocketAddr,
    db_path: PathBuf,
    log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self { 
            bind_addr: ([0, 0, 0, 0], 3000).into(),
            db_path: shellexpand::full("./releases.db3").unwrap().to_string().into(),
            log_level: "WARN".to_string()
        }
    }
}

impl Config {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Config, ()> {
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return Err(())
        };
        let mut file_contents = String::new();
        if file.read_to_string(&mut file_contents).is_err() {
            return Err(())
        }

        let toml: Config = match toml::from_str(&file_contents) {
            Ok(toml) => toml,
            Err(_) => return Err(())
        };

        if log::LevelFilter::from_str(&toml.log_level).is_err() {
            return Err(())
        } else {
            return Ok(toml)
        }
    }

    pub fn bind_addr(&self) -> &SocketAddr {
        &self.bind_addr
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    pub fn log_level(&self) -> log::LevelFilter {
        log::LevelFilter::from_str(&self.log_level).unwrap()
    }
}