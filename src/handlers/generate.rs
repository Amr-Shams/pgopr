/*
 * Eclipse Public License - v 2.0
 *
 *   THE ACCOMPANYING PROGRAM IS PROVIDED UNDER THE TERMS OF THIS ECLIPSE
 *   PUBLIC LICENSE ("AGREEMENT"). ANY USE, REPRODUCTION OR DISTRIBUTION
 *   OF THE PROGRAM CONSTITUTES RECIPIENT'S ACCEPTANCE OF THIS AGREEMENT.
 */

use crate::workload::{DeploymentConfig, PG18_PRIMARY_IMAGE, PG18_REPLICA_IMAGE};
use crate::{crd, persistent, primary, replica, services};
use clap::ArgMatches;
use std::fs;

/// Handles the 'generate' subcommand logic for various resource types.
///
/// # Arguments:
/// - `sub_matches` - The arguments passed to the generate subcommand.
pub fn handle_generate(sub_matches: &ArgMatches) {
    let config = crate::local_config::load_config();
    match sub_matches.get_one::<String>("type").unwrap().as_str() {
        "crd" => {
            crd::crd_generate();
        }
        "service" => {
            let s = services::build(&config.cluster_name, &config.namespace, 5432);
            let filename = format!("{}-service.yaml", config.cluster_name);
            let data = serde_yaml::to_string(&s).expect("Can't serialize service yaml");
            fs::write(&filename, data).expect("Unable to write file");
        }
        "persistent" => {
            let pv_name = format!("{}-pv-volume", config.cluster_name);
            let pvc_name = format!("{}-pv-claim", config.cluster_name);
            let pv = persistent::build_pv(
                &pv_name,
                config.default_storage,
                &config.cluster_name,
                "/tmp/kind",
                &config.cluster_name,
            );
            let pv_filename = format!("{}-pv.yaml", config.cluster_name);
            let data = serde_yaml::to_string(&pv).expect("Can't serialize pv yaml");
            fs::write(&pv_filename, data).expect("Unable to write file");

            let pvc = persistent::build_pvc(
                &pvc_name,
                &config.namespace,
                config.default_storage,
                &config.cluster_name,
            );
            let pvc_filename = format!("{}-pvc.yaml", config.cluster_name);
            let data = serde_yaml::to_string(&pvc).expect("Can't serialize pvc yaml");
            fs::write(&pvc_filename, data).expect("Unable to write file");
        }
        "primary" => {
            let p = primary::build(
                &config.cluster_name,
                &config.namespace,
                DeploymentConfig {
                    image: PG18_PRIMARY_IMAGE,
                    resources: None,
                    config_map_name: None,
                    config_hash: None,
                },
            );
            let filename = format!("{}-primary.yaml", config.cluster_name);
            let data = serde_yaml::to_string(&p).expect("Can't serialize primary yaml");
            fs::write(&filename, data).expect("Unable to write file");
        }
        "replica" => {
            let r_name = format!("{}-replica", config.cluster_name);
            let r = replica::build(
                &r_name,
                &config.cluster_name,
                &config.namespace,
                "replica1",
                DeploymentConfig {
                    image: PG18_REPLICA_IMAGE,
                    resources: None,
                    config_map_name: None,
                    config_hash: None,
                },
            );
            let filename = format!("{}-replica.yaml", config.cluster_name);
            let data = serde_yaml::to_string(&r).expect("Can't serialize replica yaml");
            fs::write(&filename, data).expect("Unable to write file");
        }
        "pgexporter" => {
            let secret_name = format!("{}-pgexporter-secret", config.cluster_name);
            let deploy_name = format!("{}-pgexporter", config.cluster_name);
            let data = serde_yaml::to_string(&crate::pgexporter::build_deployment(
                &deploy_name,
                &config.namespace,
                &config.cluster_name,
                &secret_name,
                None,
            ))
            .expect("Can't serialize exporter yaml");
            let filename = format!("{}-pgexporter.yaml", config.cluster_name);
            fs::write(&filename, data).expect("Unable to write file");
        }
        "pgexporter-mon" => {
            let deploy_name = format!("{}-pgexporter-mon", config.cluster_name);
            let svc_name = format!("{}-pgexporter", config.cluster_name);
            let m = crate::pgexporter::build_monitoring_deployment(
                &deploy_name,
                &config.namespace,
                &svc_name,
                None,
            );
            let filename = format!("{}-pgexporter-mon.yaml", config.cluster_name);
            let data = serde_yaml::to_string(&m).expect("Can't serialize exporter-mon yaml");
            fs::write(&filename, data).expect("Unable to write file");
        }
        name => {
            unreachable!("Unsupported type `{}`", name)
        }
    }
}
