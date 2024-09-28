pub mod handler;
pub mod request;
pub mod response;

pub type MethodFilter = axum::routing::MethodFilter;
/// MethodRouter with AppState
pub type MethodRouter = axum::routing::MethodRouter<()>;