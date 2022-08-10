use crate::probe::Probe;

/// Package containing the raw packet and corresponding [`Probe`] needed to correlate the two later
pub struct ProbeBundle<P> {
    pub probe: Probe,
    pub packet: P,
}
