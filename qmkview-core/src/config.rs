use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub always_on_top: bool,
    pub click_through: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 900.0,
            height: 400.0,
            x: 100.0,
            y: 100.0,
            always_on_top: true,
            click_through: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub key_size: f32,
    pub key_gap: f32,
    pub split_gap: f32,
    pub show_layer_name: bool,
    pub show_modifiers: bool,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            key_size: 50.0,
            key_gap: 5.0,
            split_gap: 40.0,
            show_layer_name: true,
            show_modifiers: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeymapConfig {
    pub keymap_path: Option<String>,
    pub layer_names: Vec<String>,
}

impl Default for KeymapConfig {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Self {
            keymap_path: Some(format!("{}/qmk_firmware/keyboards/crkbd/keymaps/jk/keymap.c", home)),
            layer_names: vec![
                "Base".to_string(),
                "Lower".to_string(),
                "Raise".to_string(),
                "Adjust".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub window: WindowConfig,
    pub visualization: VisualizationConfig,
    pub keymap: KeymapConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            window: WindowConfig::default(),
            visualization: VisualizationConfig::default(),
            keymap: KeymapConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path();

        if path.exists() {
            let contents = fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&contents)?)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let path = Self::config_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&path, contents)?;

        Ok(())
    }

    fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Path::new(&home).join(".config/qmkview/config.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.window.width, 900.0);
        assert_eq!(config.window.height, 400.0);
        assert!(config.window.always_on_top);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(config.window.width, parsed.window.width);
        assert_eq!(config.visualization.key_size, parsed.visualization.key_size);
        assert_eq!(config.keymap.layer_names.len(), parsed.keymap.layer_names.len());
    }
}
