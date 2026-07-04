/*
 * Eclipse Public License - v 2.0
 *
 *   THE ACCOMPANYING PROGRAM IS PROVIDED UNDER THE TERMS OF THIS ECLIPSE
 *   PUBLIC LICENSE ("AGREEMENT"). ANY USE, REPRODUCTION OR DISTRIBUTION
 *   OF THE PROGRAM CONSTITUTES RECIPIENT'S ACCEPTANCE OF THIS AGREEMENT.
 */

use crate::local_config::{get_config_path, load_config, save_config};
use clap::ArgMatches;
use log::{error, info};

pub async fn handle_config(sub_matches: &ArgMatches) {
    super::print_header();
    match sub_matches.subcommand() {
        Some(("show", _)) => {
            let config = load_config();
            if let Some(path) = get_config_path() {
                println!("Config file: {}", path.to_string_lossy());
            }
            println!("{:#?}", config);
        }
        Some(("set", set_matches)) => {
            let key = set_matches.get_one::<String>("key").unwrap();
            let value = set_matches.get_one::<String>("value").unwrap();
            let mut config = load_config();

            match key.as_str() {
                "cluster_name" => config.cluster_name = value.clone(),
                "namespace" => config.namespace = value.clone(),
                "default_storage" => {
                    if let Ok(val) = value.parse::<u32>() {
                        config.default_storage = val;
                    } else {
                        error!("Invalid value for default_storage: {}", value);
                        return;
                    }
                }
                "default_pgmoneta_storage" => {
                    if let Ok(val) = value.parse::<u32>() {
                        config.default_pgmoneta_storage = val;
                    } else {
                        error!("Invalid value for default_pgmoneta_storage: {}", value);
                        return;
                    }
                }
                _ => {
                    error!(
                        "Unknown configuration key: {}. Available keys: cluster_name, namespace, default_storage, default_pgmoneta_storage",
                        key
                    );
                    return;
                }
            }

            if let Err(e) = save_config(&config) {
                error!("Failed to save config: {}", e);
            } else {
                info!("Updated {} to {} locally", key, value);
            }
        }
        _ => unreachable!(),
    }
}
