use crate::TracerouteError;
use crate::probe::ProbeBundle;
use crate::protocol::Protocol;

pub trait PacketBuilderTrait<A, P> {
    fn build(
        protocol: Protocol,
        source: A,
        dest: A,
        ttl: u8,
    ) -> Result<ProbeBundle<P>, TracerouteError>;
}

pub struct PacketBuilder;
