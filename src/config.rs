use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub root_dir: String,
    pub port: u16,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let _ = dotenvy::dotenv;

        let root_dir = env::var("FILE_PI_ROOT_DIR").unwrap_or_else(|_| ".".to_string());

        let port = env::var("FILE_PI_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|_| "Invalid PORT value")
            .unwrap();

        let log_level = env::var("FILE_PI_LOGLEVEL").unwrap_or_else(|_| "info".to_string());

        Ok(Config {
            root_dir,
            port,
            log_level,
        })
    }
}
