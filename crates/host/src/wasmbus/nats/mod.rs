//! NATS implementations of wasmCloud [crate::wasmbus::Host] extension traits

/// Helper module for building a wasmCloud host with NATS as the primary transport.
pub mod builder;

/// NATS implementation of the wasmCloud control interface
pub mod ctl;

/// NATS implementation of the wasmCloud [crate::wasmbus::event::EventPublisher] extension trait,
/// sending events to the NATS message bus with a CloudEvents payload envelope.
pub mod event;

/// NATS implementation of the wasmCloud [crate::policy::PolicyManager] trait
pub mod policy;

/// NATS implementation of the [crate::wasmbus::secrets::SecretsManager] extension trait
/// for fetching encrypted secrets from a secret store.
pub mod secrets;

/// NATS implementation of the wasmCloud [crate::wasmbus::store::StoreManager] extension trait
/// using JetStream as a backing store.
pub mod store;
