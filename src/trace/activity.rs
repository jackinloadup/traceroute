use crate::probe::ProbeSent;
use crate::trace::TraceResponse;

/// Activity that is returned on the respose channel of a trace
#[derive(Debug)]
pub enum TraceActivity {
    Sent(ProbeSent),
    Response(TraceResponse),
    Masked(u8), // ttl
}
