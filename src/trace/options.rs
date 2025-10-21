use crate::protocol::Protocol;

/// Contains configuration parameters
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TraceOptions {
    /// The minimum TTL to probe
    /// 0 is invalid as it would drop before leaving the originating machine
    pub min_ttl: u8,
    /// The maximum TTL to probe. Must be greater than the minimum TTL
    pub max_ttl: u8,
    /// The inter-packet delay in milliseconds
    pub delay: u16,
    /// The probe response timeout in milliseconds
    pub timeout: u16,
    /// TTLs to skip probing. ex: skip closest known hosts
    pub mask: [bool; 32],
    /// Protocol to use for tracing
    pub protocol: Protocol,
    /// Output in dot format
    pub dot: bool,
}

impl TraceOptions {
    /// Returns empty Vec if no mask
    pub fn get_masked(&self) -> Vec<u8> {
        self.mask
            .iter()
            .enumerate()
            .filter_map(|(i, ttl)| if *ttl { Some((i + 1) as u8) } else { None })
            .collect()
    }

    // Create a list of all distances we are trying to probe
    pub fn get_ttl_range(&self) -> Vec<u8> {
        let min_ttl = self.min_ttl;
        let max_ttl = self.max_ttl;

        let len: u8 = self.mask.len() as u8;

        (min_ttl..=max_ttl)
            .into_iter()
            .filter(|ttl| {
                // Any ttls after the mask length are allowed regardless
                if ttl >= &len {
                    !self.mask[(*ttl as usize) - 1]
                } else {
                    true
                }
            })
            .collect()
    }

    pub fn mask(&mut self, ttl: u8) {
        self.mask[ttl as usize] = true;
    }
}

impl Default for TraceOptions {
    fn default() -> Self {
        Self {
            min_ttl: 1,
            max_ttl: 32,
            delay: 10,
            timeout: 300,
            mask: [false; 32],
            protocol: Protocol::default(),
            dot: false,
        }
    }
}
