use crate::prelude::TTL;
use crate::trace::TraceResponse;

/// Activity that is returned on the respose channel of a trace
#[derive(Debug)]
pub enum TraceActivity {
    // The response received from the wire
    Response(TraceResponse),
    // The masked distance in this flow
    Masked(TTL),
}
