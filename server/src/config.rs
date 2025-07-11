use {
    serde::{Deserialize, Serialize},
    std::{env, io, path::Path, str::FromStr},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub clues_path: String,
    pub log_level: LogLevel,
    pub port: usize,
    pub min_hint_seconds: u64,
    pub min_reveal_seconds: u64,
    pub min_skip_seconds: u64,
    pub state_channel_size: usize,
}

impl Config {
    pub fn read() -> Result<Self, io::Error> {
        let path = match env::var("TRH_CONFIG_PATH") {
            Ok(path) => Path::new(&path).to_path_buf(),
            Err(_) => env::current_dir()?.join("config.json"),
        };
        let string = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&string)?;
        Ok(config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct LogLevel {
    pub inner: tracing::Level,
}

impl TryFrom<String> for LogLevel {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl From<LogLevel> for String {
    fn from(value: LogLevel) -> Self {
        value.inner.to_string()
    }
}

impl FromStr for LogLevel {
    type Err = tracing::metadata::ParseLevelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        tracing::Level::from_str(s).map(|inner| Self { inner })
    }
}
