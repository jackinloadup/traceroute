use std::sync::mpsc::Sender;
use std::time::Duration;

use super::TraceResult;
use crate::probe::ProbeSent;

pub struct TraceSent {
    pub probes: Vec<ProbeSent>,
    pub timeout: Duration,
    pub activity_sender: Sender<TraceResult>,
}
