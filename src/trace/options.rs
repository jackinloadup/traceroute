use crate::protocol::Protocol;

/// Contains configuration parameters
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TraceOptions {
    /// Number of paths to probe
    pub npaths: u8,
    /// The minimum TTL to probe
    pub min_ttl: u8,
    /// The maximum TTL to probe. Must be greater than the minimum TTL
    pub max_ttl: u8,
    /// The inter-packet delay in milliseconds
    pub delay: u16,
    /// TTLs to skip probing
    pub mask: Option<Vec<u8>>,
    /// Protocol to use for tracing
    pub protocol: Protocol,
}

impl TraceOptions {
    /// Returns empty Vec if no mask
    pub fn get_masked(&self) -> Vec<u8> {
        self.mask.to_owned().unwrap_or_default()
    }

    // Create a list of all distances we are trying to probe
    pub fn get_ttl_range(&self) -> Vec<u8> {
        let range = (self.min_ttl..=self.max_ttl).into_iter();

        match &self.mask {
            // Mask out any distances we want to ignore
            Some(vec) => range.filter(|ttl| !vec.contains(ttl)).collect(),
            // No need to mask
            None => range.collect(),
        }
    }
}

impl Default for TraceOptions {
    fn default() -> Self {
        Self {
            npaths: 1,
            min_ttl: 0,
            max_ttl: 30,
            delay: 10,
            mask: None,
            protocol: Protocol::default(),
        }
    }
}
