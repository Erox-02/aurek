use std::fs;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub color: bool,
    pub llm_response_on_cli: bool,
    pub default_llm_model: Option<String>,
    pub scan_by_default: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            color: true,
            llm_response_on_cli: false,
            default_llm_model: None,
            scan_by_default: true,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_paths = vec![
            PathBuf::from("/etc/aurek.conf"),
        ];

        for path in config_paths {
            if path.exists() {
                return Self::from_file(&path);
            }
        }

        Ok(Self::default())
    }

    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let mut config = Config::default();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_end_matches(';');

                match key {
                    "color" => config.color = value == "1" || value == "true",
                    "llm_response_on_cli" => config.llm_response_on_cli = value == "1" || value == "true",
                    "default_llm_model" => config.default_llm_model = Some(value.to_string()),
                    "scan_by_default" => config.scan_by_default = value == "1" || value == "true",
                    _ => {}
                }
            }
        }

        Ok(config)
    }
}