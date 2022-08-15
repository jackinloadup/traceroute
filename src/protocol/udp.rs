#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UdpParams {
    /// Port to send packets from
    ///
    /// Source port can be randomized or carry identifying information to be echoed back
    pub source_port: u16,
    /// Port to send packets to
    ///
    /// Destination port is typically 33434 to 33464. Info/spec source needed
    pub destination_port: u16,
}

impl Default for UdpParams {
    fn default() -> Self {
        Self {
            source_port: 33434,
            destination_port: 33434,
        }
    }
}
