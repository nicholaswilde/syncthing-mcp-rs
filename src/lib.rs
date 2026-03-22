pub mod api;
pub mod config;
pub mod credentials;
pub mod error;
#[cfg(test)]
mod error_tests;
pub mod mcp;
#[cfg(test)]
pub mod test_utils;
pub mod tools;

pub use error::{Error, Result};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(version(), "0.1.0");
    }
}
