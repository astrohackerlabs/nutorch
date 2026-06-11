//! nutorchd as a library: everything except the socket/lifecycle wiring,
//! exposed so integration tests (notably the golden harness, issue 0005)
//! can drive dispatch in-process.

pub mod convert;
pub mod dispatch;
pub mod lifecycle;
pub mod nn;
pub mod protocol;
pub mod registry;
