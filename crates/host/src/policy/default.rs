//! Policy manager implementation that uses default implementations (allow all requests)

/// A default policy manager that always returns true for all requests
/// This is used when no policy manager is configured
pub struct DefaultPolicyManager;
impl DefaultPolicyManager {
    /// Create a new default policy manager
    pub fn new() -> Self {
        Self {}
    }
}
impl super::PolicyManager for DefaultPolicyManager {}
