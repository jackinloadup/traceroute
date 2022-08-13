use crate::probe::ProbeBundle;
use crate::utils::Protocol;
use crate::TracerouteError;

pub trait PacketBuilderTrait<A, P> {
    fn build(
        protocol: Protocol,
        source: A,
        source_port: u16,
        dest: A,
        dest_port: u16,
        ttl: u8,
    ) -> Result<ProbeBundle<P>, TracerouteError>;
}

pub struct PacketBuilder;
