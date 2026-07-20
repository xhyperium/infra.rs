//! `infra-core` — 核心基础设施库

pub mod error;

pub use error::{Error, Result};

/// Returns a greeting.
pub fn hello() -> &'static str {
    "Hello from infra-core"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        assert_eq!(hello(), "Hello from infra-core");
    }

    #[test]
    fn test_error_conversion() {
        let err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let infra_err: Error = err.into();
        assert!(matches!(infra_err, Error::Io(_)));
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            Error::InvalidArgument("missing field".into()).to_string(),
            "Invalid argument: missing field"
        );
    }

    #[test]
    fn test_result_alias() {
        fn returns_result() -> Result<i32> {
            Err(Error::Config("test".into()))
        }
        assert!(returns_result().is_err());
    }
}
