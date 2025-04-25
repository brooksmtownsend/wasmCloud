//! NATS implementations of wasmCloud [crate::wasmbus::Host] extension traits

/// NATS implementation of the wasmCloud control interface
pub mod ctl;

/// NATS implementation of the wasmCloud [crate::policy::PolicyManager] trait
pub mod policy;

/// NATS implementation of the wasmCloud [crate::wasmbus::event::EventPublisher] extension trait,
/// sending events to the NATS message bus with a CloudEvents payload envelope.
pub mod event;
