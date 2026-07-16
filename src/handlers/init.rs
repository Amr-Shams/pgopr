/*
 * Eclipse Public License - v 2.0
 *
 *   THE ACCOMPANYING PROGRAM IS PROVIDED UNDER THE TERMS OF THIS ECLIPSE
 *   PUBLIC LICENSE ("AGREEMENT"). ANY USE, REPRODUCTION OR DISTRIBUTION
 *   OF THE PROGRAM CONSTITUTES RECIPIENT'S ACCEPTANCE OF THIS AGREEMENT.
 */

use crate::local_config::{LocalConfig, save_config};
use std::io::{BufRead, Write};

fn prompt_default(label: &str, default: &str) -> String {
    print!("{label} [{default}]: ");
    std::io::stdout().flush().ok();
    let mut input = String::new();
    std::io::stdin().lock().read_line(&mut input).ok();
    let trimmed = input.trim().to_string();
    if trimmed.is_empty() {
        default.to_string()
    } else {
        trimmed
    }
}

fn prompt_u32(label: &str, default: u32) -> u32 {
    loop {
        print!("{label} [{default}]: ");
        std::io::stdout().flush().ok();
        let mut input = String::new();
        std::io::stdin().lock().read_line(&mut input).ok();
        let trimmed = input.trim().to_string();
        if trimmed.is_empty() {
            return default;
        }
        match trimmed.parse::<u32>() {
            Ok(val) => return val,
            Err(_) => println!("Invalid number, try again"),
        }
    }
}

fn confirm(prompt: &str) -> bool {
    print!("{prompt} [Y/n]: ");
    std::io::stdout().flush().ok();
    let mut input = String::new();
    std::io::stdin().lock().read_line(&mut input).ok();
    let trimmed = input.trim().to_lowercase();
    matches!(trimmed.as_str(), "" | "y" | "yes")
}

pub async fn handle_init() {
    println!("pgopr configuration");
    println!();

    let cluster_name = prompt_default("Cluster name", "postgresql");
    let namespace = prompt_default("Namespace", "default");
    let default_storage = prompt_u32("Default storage (GiB)", 5);
    let default_pgmoneta_storage = prompt_u32("pgmoneta storage (GiB)", 10);

    let config = LocalConfig {
        cluster_name,
        namespace,
        default_storage,
        default_pgmoneta_storage,
    };

    println!();
    println!("Configuration preview:");
    println!("{:#?}", config);
    println!();

    if confirm("Write this configuration?") {
        match save_config(&config) {
            Ok(()) => println!("Configuration written to ~/.pgopr/pgopr.toml"),
            Err(e) => eprintln!("Failed to save configuration: {e}"),
        }
    } else {
        println!("Configuration not written");
    }
}
