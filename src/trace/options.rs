/// Contains configuration parameters
#[derive(Clone, Debug)]
pub struct TraceOptions {
    /// Source port to send packets from
    pub src_port: u16,
    /// Base destination port to send packets to
    pub dst_port: u16,
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
}

impl TraceOptions {
    /// Returns empty Vec if no mask
    pub fn get_masked(&self) -> Vec<u8> {
        self.mask.to_owned().unwrap_or_default()
    }
}

impl Default for TraceOptions {
    fn default() -> Self {
        Self {
            src_port: 12345,
            dst_port: 33434,
            npaths: 1,
            min_ttl: 0,
            max_ttl: 30,
            delay: 10,
            mask: None,
        }
    }
}