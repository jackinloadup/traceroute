use crate::probe::{ProbeResponse, ProbeSent};

/// The outcome of any probe we handled
#[derive(Debug)]
pub enum TraceResponse {
    /// The sent probe responded
    Received(ProbeResponse),
    /// The sent probe was ignored or didn't make it back
    TimedOut(ProbeSent),
}
