use crate::probe::{ProbeResponse, TcpId};

/// The outcome of any probe we handled
#[derive(Debug)]
pub enum TraceResponse {
    /// The sent probe responded
    Found(ProbeResponse),
    /// The sent probe was ignored or didn't make it back
    NotReceived(TcpId),
}

impl TraceResponse {
    pub fn get_id(&self) -> &TcpId {
        match self {
            Self::Found(respose) => respose.get_id(),
            Self::NotReceived(id) => id,
        }
    }
}
