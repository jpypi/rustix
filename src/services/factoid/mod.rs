mod factoid;
mod backend;
mod del_factoid;
pub(in crate::services) mod models;

// Re-export
pub use factoid::Factoid;
pub use del_factoid::DelFactoid;
