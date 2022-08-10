use crate::probe::ProbeResponse;

/// The outcome of any probe we handled
#[derive(Debug)]
pub enum TraceResponse {
    /// The sent probe responded
    Found(ProbeResponse),
    /// The sent probe was ignored or didn't make it back
    NotReceived(u16),
}

impl TraceResponse {
    pub fn get_id(&self) -> &u16 {
        match self {
            Self::Found(respose) => respose.get_id(),
            Self::NotReceived(id) => id,
        }
    }
}
