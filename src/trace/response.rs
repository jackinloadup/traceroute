use std::net::IpAddr;

use crate::probe::{ProbeResponse, ProbeSent};

/// The outcome of any probe we handled
#[derive(Clone,Debug)]
pub enum TraceResponse {
    /// The sent probe responded
    Received(ProbeResponse),
    /// The sent probe was ignored or didn't make it back
    TimedOut(ProbeSent),
    /// Masked
    Masked(u8),
}

impl TraceResponse {
    pub fn get_distance(&self) -> u8 {
        match self {
            Self::Received(response) => response.ttl,
            Self::TimedOut(sent) => sent.ttl,
            Self::Masked(ttl) => ttl.clone(),
        }
    }

    pub fn get_destination(&self) -> Option<IpAddr> {
        match self {
            Self::Received(response) => Some(response.destination),
            Self::TimedOut(_sent) => None,
            Self::Masked(_ttl) => None,
        }
    }
}
