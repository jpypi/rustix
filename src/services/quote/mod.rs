mod backend;
mod models;

mod quotes;
mod del_quote;
mod edit_quote;

// Re-export
pub use quotes::Quotes;
pub use del_quote::DelQuote;
pub use edit_quote::EditQuote;
