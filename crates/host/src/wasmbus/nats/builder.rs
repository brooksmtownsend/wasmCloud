//! An opinionated [crate::wasmbus::HostBuilder] that uses NATS as the primary transport.

use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::Duration,
};

use anyhow::ensure;
use async_nats::Client;
use nkeys::KeyPair;
use wasmcloud_core::RegistryConfig;

use crate::{
    oci,
    secrets::SecretsManager,
    wasmbus::{
        config::BundleGenerator,
        event::EventPublisher,
        jetstream::create_bucket,
        load_supplemental_config, merge_registry_config,
        nats::{event::NatsEventPublisher, policy::NatsPolicyManager, secrets::NatsSecretsManager},
        store::StoreManager,
        HostBuilder, SupplementalConfig,
    },
    PolicyHostInfo, PolicyManager, WasmbusHostConfig,
};

pub struct NatsHostBuilder {
    // Required fields
    ctl_nats: Client,
    ctl_topic_prefix: String,
    config_generator: BundleGenerator,
    registry_config: HashMap<String, RegistryConfig>,

    // Trait implementations for NATS
    config_store: Arc<dyn StoreManager>,
    data_store: Arc<dyn StoreManager>,
    policy_manager: Option<Arc<dyn PolicyManager>>,
    secrets_manager: Option<Arc<dyn SecretsManager>>,
    event_publisher: Option<Arc<dyn EventPublisher>>,
}

impl NatsHostBuilder {
    /// Initialize the host with the NATS control interface connection
    ///
    pub async fn new(
        ctl_nats: Client,
        ctl_topic_prefix: String,
        lattice: String,
        js_domain: Option<String>,
        config_service_enabled: bool,
        oci_opts: oci::Config,
        labels: BTreeMap<String, String>,
    ) -> anyhow::Result<Self> {
        let ctl_jetstream = if let Some(domain) = js_domain.as_ref() {
            async_nats::jetstream::with_domain(ctl_nats.clone(), domain)
        } else {
            async_nats::jetstream::new(ctl_nats.clone())
        };
        let bucket = format!("LATTICEDATA_{}", lattice);
        let data = create_bucket(&ctl_jetstream, &bucket).await?;

        let config_bucket = format!("CONFIGDATA_{}", lattice);
        let config_data = create_bucket(&ctl_jetstream, &config_bucket).await?;

        let supplemental_config = if config_service_enabled {
            load_supplemental_config(&ctl_nats, &lattice, &labels).await?
        } else {
            SupplementalConfig::default()
        };

        let mut registry_config = supplemental_config.registry_config.unwrap_or_default();
        merge_registry_config(&mut registry_config, oci_opts).await;

        let config_generator = BundleGenerator::new(config_data.clone());

        Ok(Self {
            ctl_nats,
            ctl_topic_prefix,
            config_generator,
            registry_config,
            config_store: Arc::new(config_data),
            data_store: Arc::new(data),
            policy_manager: None,
            secrets_manager: None,
            event_publisher: None,
        })
    }

    /// Setup the NATS policy manager for the host
    pub async fn with_policy_manager(
        self,
        host_key: KeyPair,
        lattice: String,
        labels: HashMap<String, String>,
        policy_topic: Option<String>,
        policy_timeout: Option<Duration>,
        policy_changes_topic: Option<String>,
    ) -> anyhow::Result<Self> {
        let policy_manager = NatsPolicyManager::new(
            self.ctl_nats.clone(),
            PolicyHostInfo {
                public_key: host_key.public_key(),
                lattice,
                labels,
            },
            policy_topic,
            policy_timeout,
            policy_changes_topic,
        )
        .await?;

        Ok(NatsHostBuilder {
            policy_manager: Some(Arc::new(policy_manager)),
            ..self
        })
    }

    /// Setup the NATS secrets manager for the host
    pub async fn with_secrets_manager(self, secrets_topic_prefix: String) -> anyhow::Result<Self> {
        ensure!(
            !secrets_topic_prefix.is_empty(),
            "secrets topic prefix must be non-empty"
        );
        let secrets_manager = NatsSecretsManager::new(
            Arc::clone(&self.config_store),
            Some(&secrets_topic_prefix),
            &self.ctl_nats,
        );

        Ok(NatsHostBuilder {
            secrets_manager: Some(Arc::new(secrets_manager)),
            ..self
        })
    }

    /// Setup the NATS event publisher for the host
    pub async fn with_event_publisher(
        self,
        host_key: KeyPair,
        lattice: String,
    ) -> anyhow::Result<Self> {
        let event_publisher =
            NatsEventPublisher::new(host_key.public_key(), lattice, self.ctl_nats.clone());

        Ok(NatsHostBuilder {
            event_publisher: Some(Arc::new(event_publisher)),
            ..self
        })
    }

    /// Build the [`HostBuilder`] with the NATS extension traits and the provided [`WasmbusHostConfig`].
    pub async fn build(self, config: WasmbusHostConfig) -> anyhow::Result<HostBuilder> {
        Ok(HostBuilder::from(config)
            .with_config_store(Some(self.config_store))
            .with_data_store(Some(self.data_store))
            .with_registry_config(self.registry_config)
            .with_event_publisher(self.event_publisher)
            .with_policy_manager(self.policy_manager)
            .with_secrets_manager(self.secrets_manager))
    }
}
