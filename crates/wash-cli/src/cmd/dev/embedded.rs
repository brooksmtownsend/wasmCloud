use std::sync::Arc;

use anyhow::Context as _;
use nkeys::KeyPair;
use wasmcloud_host::wasmbus::Features;

/// Start a wasmCloud host in process
pub async fn start_host() -> anyhow::Result<String> {
    let host_key = Arc::new(KeyPair::new_server());
    let host_config = wasmcloud_host::WasmbusHostConfig {
        host_key: Some(host_key.clone()),
        allow_file_load: true,
        experimental_features: Features::new()
            .enable_builtin_http_server()
            .enable_builtin_messaging_nats()
            .enable_wasmcloud_messaging_v3(),
        ..Default::default()
    };
    let (_host, _shutdown) = wasmcloud_host::WasmbusHost::new(host_config).await?;
    Ok(host_key.public_key())
}

/// Start wadm in process
pub async fn start_wadm() -> anyhow::Result<()> {
    let wadm_config = wadm::config::WadmConfig {
        stream_persistence: wadm::StreamPersistence::Memory,
        ..Default::default()
    };
    let joinset = wadm::start_wadm(wadm_config)
        .await
        .context("failed to start wadm")?;

    // TODO: Return joinset to await later
    tokio::spawn(joinset.join_all());
    Ok(())
}

/// Start NATS from a preinstalled binary
// pub async fn start_nats() -> anyhow::Result<()> {
//     tokio::process::Command::new("nats-server")
//         .arg("-js")
//         .spawn()?;

//     tokio::time::sleep(std::time::Duration::from_secs(1)).await;

//     Ok(())
// }

/// Start NATS from linked library
// #[cfg(feature = "embedded_nats")]
pub async fn start_nats() -> anyhow::Result<()> {
    let server = nats_server::NatsServer::new();

    println!("Running NATS from Rust!");
    server.start();

    Ok(())
}
