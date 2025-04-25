//! Implementation of the [ConfigManager] trait for NATS JetStream KV [Store].

use async_nats::jetstream::kv::Store;
use bytes::Bytes;
use tracing::instrument;

use crate::wasmbus::store::StoreManager;

// pub async fn data_watch() {
//     // TODO: Move this to the NATS version
//     let data_watch: JoinHandle<anyhow::Result<_>> = spawn({
//         let host = Arc::clone(&host);
//         let data = host.data_store.clone();
//         async move {
//             // Setup data watch first
//             let data_watch = data
//                 .watch_all()
//                 .await
//                 .context("failed to watch lattice data bucket")?;

//             // Process existing data without emitting events
//             data.keys()
//                 .await
//                 .context("failed to read keys of lattice data bucket")?
//                 .map_err(|e| anyhow!(e).context("failed to read lattice data stream"))
//                 .try_filter_map(|key| async {
//                     data.entry(key)
//                         .await
//                         .context("failed to get entry in lattice data bucket")
//                 })
//                 .for_each(|entry| async {
//                     match entry {
//                         Ok(entry) => host.process_entry(entry).await,
//                         Err(err) => {
//                             error!(%err, "failed to read entry from lattice data bucket")
//                         }
//                     }
//                 })
//                 .await;
//             let mut data_watch = Abortable::new(data_watch, data_watch_abort_reg);
//             data_watch
//                 .by_ref()
//                 .for_each({
//                     let host = Arc::clone(&host);
//                     move |entry| {
//                         let host = Arc::clone(&host);
//                         async move {
//                             match entry {
//                                 Err(error) => {
//                                     error!("failed to watch lattice data bucket: {error}");
//                                 }
//                                 Ok(entry) => host.process_entry(entry).await,
//                             }
//                         }
//                     }
//                 })
//                 .await;
//             let deadline = { *host.stop_rx.borrow() };
//             host.stop_tx.send_replace(deadline);
//             if data_watch.is_aborted() {
//                 info!("data watch task gracefully stopped");
//             } else {
//                 error!("data watch task unexpectedly stopped");
//             }
//             Ok(())
//         }
//     });
// }

#[async_trait::async_trait]
impl StoreManager for Store {
    #[instrument(level = "debug", skip(self))]
    async fn get(&self, key: &str) -> anyhow::Result<Option<Bytes>> {
        self.get(key)
            .await
            .map_err(|err| anyhow::anyhow!("Failed to get config: {}", err))
    }

    #[instrument(level = "debug", skip(self, value))]
    async fn put(&self, key: &str, value: Bytes) -> anyhow::Result<()> {
        self.put(key, value)
            .await
            .map(|_| ())
            .map_err(|err| anyhow::anyhow!("Failed to set config: {}", err))
    }

    #[instrument(level = "debug", skip(self))]
    async fn del(&self, key: &str) -> anyhow::Result<()> {
        self.purge(key)
            .await
            .map_err(|err| anyhow::anyhow!("Failed to delete config: {}", err))
    }
}
