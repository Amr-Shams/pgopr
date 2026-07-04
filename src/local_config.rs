/*
 * Eclipse Public License - v 2.0
 *
 *   THE ACCOMPANYING PROGRAM IS PROVIDED UNDER THE TERMS OF THIS ECLIPSE
 *   PUBLIC LICENSE ("AGREEMENT"). ANY USE, REPRODUCTION OR DISTRIBUTION
 *   OF THE PROGRAM CONSTITUTES RECIPIENT'S ACCEPTANCE OF THIS AGREEMENT.
 */

use directories::BaseDirs;
use figment::{
    Figment,
    providers::{Format, Toml},
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalConfig {
    #[serde(default = "default_cluster_name")]
    pub cluster_name: String,
    #[serde(default = "default_namespace")]
    pub namespace: String,
    #[serde(default = "default_storage")]
    pub default_storage: u32,
    #[serde(default = "default_pgmoneta_storage")]
    pub default_pgmoneta_storage: u32,
}

fn default_cluster_name() -> String {
    "postgresql".to_string()
}
fn default_namespace() -> String {
    "default".to_string()
}
fn default_storage() -> u32 {
    5
}
fn default_pgmoneta_storage() -> u32 {
    10
}

impl Default for LocalConfig {
    fn default() -> Self {
        LocalConfig {
            cluster_name: default_cluster_name(),
            namespace: default_namespace(),
            default_storage: default_storage(),
            default_pgmoneta_storage: default_pgmoneta_storage(),
        }
    }
}

pub fn get_config_path() -> Option<PathBuf> {
    BaseDirs::new().map(|base| base.home_dir().join(".pgopr").join("pgopr.toml"))
}

pub fn load_config() -> LocalConfig {
    get_config_path()
        .filter(|path| path.exists())
        .and_then(|path| {
            Figment::new()
                .merge(Toml::file(&path))
                .extract::<LocalConfig>()
                .ok()
        })
        .unwrap_or_default()
}

pub fn save_config(config: &LocalConfig) -> Result<(), String> {
    let path = get_config_path().ok_or_else(|| "Could not find home directory".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    let toml_string =
        toml::to_string_pretty(config).map_err(|e| format!("Serialization error: {}", e))?;
    fs::write(&path, toml_string).map_err(|e| format!("Failed to write config file: {}", e))?;
    Ok(())
}
