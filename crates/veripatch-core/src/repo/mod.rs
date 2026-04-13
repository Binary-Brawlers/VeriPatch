//! Repository preparation helpers for verification.

mod git;
mod prepare;

pub use git::load_local_diff;
pub use prepare::VerificationMode;
pub(crate) use prepare::prepare_repository;
