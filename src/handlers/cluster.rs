/*
 * Eclipse Public License - v 2.0
 *
 *   THE ACCOMPANYING PROGRAM IS PROVIDED UNDER THE TERMS OF THIS ECLIPSE
 *   PUBLIC LICENSE ("AGREEMENT"). ANY USE, REPRODUCTION OR DISTRIBUTION
 *   OF THE PROGRAM CONSTITUTES RECIPIENT'S ACCEPTANCE OF THIS AGREEMENT.
 */
use crate::crd::v1::PgOprSpec;
use crate::crd::v1::pgopr;
use crate::k8s;
use crate::local_config::load_config;
use kube::{
    Api, Client,
    api::{DeleteParams, Patch, PatchParams, PostParams},
};
use log::error;

/// Orchestrates the installation of the operator and its CRDs.
pub async fn handle_install() {
    super::print_header();
    let client: Client = k8s::k8s_client().await;
    let _ = crate::crd::crd_deploy(client).await;
}

/// Orchestrates the uninstallation of the operator and its CRDs.
pub async fn handle_uninstall() {
    super::print_header();
    let client: Client = k8s::k8s_client().await;
    let _ = crate::crd::crd_undeploy(client).await;
}

/// Creates the PgOpr resource.
///
/// # Arguments
/// - `client` - Kubernetes client to create the PgOpr resource with.
/// - `name` - Name of the PgOpr resource to create.
/// - `namespace` - Namespace where the PgOpr resource resides.
/// - `replicas` - Desired number of replicas.
/// - `storage` - Desired storage size.
async fn create_cluster(
    client: Client,
    name: &str,
    namespace: &str,
    replicas: u32,
    storage: u32,
) -> Result<pgopr, crate::Error> {
    let api: Api<pgopr> = Api::namespaced(client, namespace);
    let mut cluster = pgopr::new(
        name,
        PgOprSpec {
            version: None,
            storage,
            replicas: Some(replicas),
            resources: None,
            config: None,
            pgmoneta: None,
            pgexporter: None,
        },
    );
    cluster.metadata.namespace = Some(namespace.to_string());

    api.create(&PostParams::default(), &cluster)
        .await
        .map_err(crate::Error::from)
}

/// Updates the replica count on an existing PgOpr resource.
///
/// # Arguments
/// - `client` - Kubernetes client to modify the PgOpr resource with.
/// - `name` - Name of the PgOpr resource to modify.
/// - `namespace` - Namespace where the PgOpr resource resides.
/// - `replicas` - Desired number of replicas.
async fn patch_replicas(
    client: Client,
    name: &str,
    namespace: &str,
    replicas: u32,
) -> Result<pgopr, crate::Error> {
    let api: Api<pgopr> = Api::namespaced(client, namespace);
    let patch = serde_json::json!({
        "spec": {
            "replicas": replicas
        }
    });

    api.patch(name, &PatchParams::default(), &Patch::Merge(&patch))
        .await
        .map_err(crate::Error::from)
}

/// Gets the PgOpr resource.
///
/// # Arguments
/// - `client` - Kubernetes client to get the PgOpr resource with.
/// - `name` - Name of the PgOpr resource to get.
/// - `namespace` - Namespace where the PgOpr resource resides.
async fn get_cluster(client: Client, name: &str, namespace: &str) -> Result<pgopr, crate::Error> {
    let api: Api<pgopr> = Api::namespaced(client, namespace);
    api.get(name).await.map_err(crate::Error::from)
}

/// Provisions the primary database components through a PgOpr resource.
pub async fn handle_provision_primary() {
    super::print_header();
    let local_cfg = load_config();
    let client: Client = k8s::k8s_client().await;
    match get_cluster(
        client.clone(),
        &local_cfg.cluster_name,
        &local_cfg.namespace,
    )
    .await
    {
        Ok(_) => {}
        Err(crate::Error::KubeError { source }) => match source {
            kube::Error::Api(err) if err.code == 404 => {
                if let Err(err) = create_cluster(
                    client,
                    &local_cfg.cluster_name,
                    &local_cfg.namespace,
                    0,
                    local_cfg.default_storage,
                )
                .await
                {
                    error!("Unable to create PgOpr resource: {:?}", err);
                }
            }
            err => error!("Unable to get PgOpr resource: {:?}", err),
        },
        Err(err) => error!("Unable to get PgOpr resource: {:?}", err),
    }
}

/// Provisions pgmoneta through the PgOpr resource.
pub async fn handle_provision_pgmoneta() {
    super::print_header();
    let local_cfg = load_config();
    let client: Client = k8s::k8s_client().await;
    match get_cluster(
        client.clone(),
        &local_cfg.cluster_name,
        &local_cfg.namespace,
    )
    .await
    {
        Ok(_) => {
            let api: Api<pgopr> = Api::namespaced(client, &local_cfg.namespace);
            let patch = serde_json::json!({
                "spec": {
                    "pgmoneta": {
                        "storage": local_cfg.default_pgmoneta_storage
                    }
                }
            });
            if let Err(err) = api
                .patch(
                    &local_cfg.cluster_name,
                    &PatchParams::default(),
                    &Patch::Merge(&patch),
                )
                .await
            {
                error!("Unable to patch PgOpr pgmoneta: {:?}", err);
            }
        }
        Err(crate::Error::KubeError { source }) => match source {
            kube::Error::Api(err) if err.code == 404 => {
                if let Err(err) = create_cluster(
                    client,
                    &local_cfg.cluster_name,
                    &local_cfg.namespace,
                    0,
                    local_cfg.default_storage,
                )
                .await
                {
                    error!("Unable to create PgOpr resource: {:?}", err);
                }
            }
            err => error!("Unable to get PgOpr resource: {:?}", err),
        },
        Err(err) => error!("Unable to get PgOpr resource: {:?}", err),
    }
}

/// Provisions pgexporter through the PgOpr resource.
pub async fn handle_provision_pgexporter() {
    super::print_header();
    let local_cfg = load_config();
    let client: Client = k8s::k8s_client().await;
    match get_cluster(
        client.clone(),
        &local_cfg.cluster_name,
        &local_cfg.namespace,
    )
    .await
    {
        Ok(_) => {
            let api: Api<pgopr> = Api::namespaced(client, &local_cfg.namespace);
            let patch = serde_json::json!({
                "spec": {
                    "pgexporter": {}
                }
            });
            if let Err(err) = api
                .patch(
                    &local_cfg.cluster_name,
                    &PatchParams::default(),
                    &Patch::Merge(&patch),
                )
                .await
            {
                error!("Unable to patch PgOpr pgexporter: {:?}", err);
            }
        }
        Err(crate::Error::KubeError { source }) => match source {
            kube::Error::Api(err) if err.code == 404 => {
                if let Err(err) = create_cluster(
                    client,
                    &local_cfg.cluster_name,
                    &local_cfg.namespace,
                    0,
                    local_cfg.default_storage,
                )
                .await
                {
                    error!("Unable to create PgOpr resource: {:?}", err);
                }
            }
            err => error!("Unable to get PgOpr resource: {:?}", err),
        },
        Err(err) => error!("Unable to get PgOpr resource: {:?}", err),
    }
}

/// Retires pgexporter through the PgOpr resource.
pub async fn handle_retire_pgexporter() {
    super::print_header();
    let local_cfg = load_config();
    let client: Client = k8s::k8s_client().await;
    let api: Api<pgopr> = Api::namespaced(client, &local_cfg.namespace);
    let patch = serde_json::json!({
        "spec": {
            "pgexporter": null
        }
    });
    if let Err(err) = api
        .patch(
            &local_cfg.cluster_name,
            &PatchParams::default(),
            &Patch::Merge(&patch),
        )
        .await
    {
        error!("Unable to patch PgOpr pgexporter: {:?}", err);
    }
}

/// Retires pgmoneta through the PgOpr resource.
pub async fn handle_retire_pgmoneta() {
    super::print_header();
    let local_cfg = load_config();
    let client: Client = k8s::k8s_client().await;
    let api: Api<pgopr> = Api::namespaced(client, &local_cfg.namespace);
    let patch = serde_json::json!({
        "spec": {
            "pgmoneta": null
        }
    });
    if let Err(err) = api
        .patch(
            &local_cfg.cluster_name,
            &PatchParams::default(),
            &Patch::Merge(&patch),
        )
        .await
    {
        error!("Unable to patch PgOpr pgmoneta: {:?}", err);
    }
}

/// Provisions pgexporter monitoring (Grafana + Prometheus) through the PgOpr resource.
pub async fn handle_provision_grafana() {
    super::print_header();
    let local_cfg = load_config();
    let client: Client = k8s::k8s_client().await;
    match get_cluster(
        client.clone(),
        &local_cfg.cluster_name,
        &local_cfg.namespace,
    )
    .await
    {
        Ok(_) => {
            let api: Api<pgopr> = Api::namespaced(client, &local_cfg.namespace);
            let patch = serde_json::json!({
                "spec": {
                    "pgexporter": {
                        "monitoring": {}
                    }
                }
            });
            if let Err(err) = api
                .patch(
                    &local_cfg.cluster_name,
                    &PatchParams::default(),
                    &Patch::Merge(&patch),
                )
                .await
            {
                error!("Unable to patch PgOpr pgexporter monitoring: {:?}", err);
            }
        }
        Err(crate::Error::KubeError { source }) => match source {
            kube::Error::Api(err) if err.code == 404 => {
                error!(
                    "PgOpr resource does not exist. Create the cluster first with `pgopr provision primary`"
                );
            }
            err => error!("Unable to get PgOpr resource: {:?}", err),
        },
        Err(err) => error!("Unable to get PgOpr resource: {:?}", err),
    }
}

/// Retires pgexporter monitoring (Grafana + Prometheus) through the PgOpr resource.
pub async fn handle_retire_grafana() {
    super::print_header();
    let local_cfg = load_config();
    let client: Client = k8s::k8s_client().await;
    let api: Api<pgopr> = Api::namespaced(client, &local_cfg.namespace);
    let patch = serde_json::json!({
        "spec": {
            "pgexporter": {
                "monitoring": null
            }
        }
    });
    if let Err(err) = api
        .patch(
            &local_cfg.cluster_name,
            &PatchParams::default(),
            &Patch::Merge(&patch),
        )
        .await
    {
        error!("Unable to patch PgOpr pgexporter monitoring: {:?}", err);
    }
}

/// Removes the primary database components through the PgOpr resource.
pub async fn handle_retire_primary() {
    super::print_header();
    let local_cfg = load_config();
    let client: Client = k8s::k8s_client().await;
    let api: Api<pgopr> = Api::namespaced(client, &local_cfg.namespace);

    match api
        .delete(&local_cfg.cluster_name, &DeleteParams::default())
        .await
    {
        Ok(_) => {}
        Err(kube::Error::Api(err)) if err.code == 404 => {}
        Err(err) => {
            error!("Unable to delete PgOpr resource: {:?}", err);
        }
    }
}

/// Provisions the replica database components through a PgOpr resource.
pub async fn handle_provision_replica() {
    super::print_header();
    let local_cfg = load_config();
    let client: Client = k8s::k8s_client().await;
    match get_cluster(
        client.clone(),
        &local_cfg.cluster_name,
        &local_cfg.namespace,
    )
    .await
    {
        Ok(current) => {
            let replicas = current.spec.replicas.unwrap_or(0) + 1;
            if let Err(err) = patch_replicas(
                client,
                &local_cfg.cluster_name,
                &local_cfg.namespace,
                replicas,
            )
            .await
            {
                error!("Unable to patch PgOpr replicas: {:?}", err);
            }
        }
        Err(crate::Error::KubeError { source }) => match source {
            kube::Error::Api(err) if err.code == 404 => {
                if let Err(err) = create_cluster(
                    client,
                    &local_cfg.cluster_name,
                    &local_cfg.namespace,
                    1,
                    local_cfg.default_storage,
                )
                .await
                {
                    error!("Unable to create PgOpr resource: {:?}", err);
                }
            }
            err => error!("Unable to get PgOpr resource: {:?}", err),
        },
        Err(err) => error!("Unable to get PgOpr resource: {:?}", err),
    }
}

/// Removes the replica database components through a PgOpr resource.
pub async fn handle_retire_replica() {
    super::print_header();
    let local_cfg = load_config();
    let client: Client = k8s::k8s_client().await;
    match get_cluster(
        client.clone(),
        &local_cfg.cluster_name,
        &local_cfg.namespace,
    )
    .await
    {
        Ok(current) => {
            let replicas = current.spec.replicas.unwrap_or(0).saturating_sub(1);
            if let Err(err) = patch_replicas(
                client,
                &local_cfg.cluster_name,
                &local_cfg.namespace,
                replicas,
            )
            .await
            {
                error!("Unable to patch PgOpr replicas: {:?}", err);
            }
        }
        Err(err) => error!("Unable to get PgOpr resource: {:?}", err),
    }
}
