mod activity;
mod options;
mod response;
mod trace;

use crate::TracerouteError;

pub type TraceResult = Result<TraceActivity, TracerouteError>;

pub use activity::TraceActivity;
pub use options::TraceOptions;
pub use response::TraceResponse;
pub use trace::Trace;
