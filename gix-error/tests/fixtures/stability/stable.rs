use gix_error::{Error, Result};

fn main() -> Result {
    fn standard_error<E: std::error::Error + Send + Sync + 'static>(_: &E) {}
    let error = Error::from_error(std::io::Error::from(std::io::ErrorKind::PermissionDenied));
    standard_error(&error);
    assert!(
        error.is_permission_denied(),
        "native authorization failures retain their recovery class"
    );
    assert!(!error.is_retryable(), "authorization failure is not lock contention");
    assert_eq!(error.iter_errors().count(), 1, "native errors remain inspectable");
    assert!(error.error().is::<std::io::Error>(), "the concrete root survives");
    assert_eq!(
        error
            .downcast_any_ref::<std::io::Error>()
            .expect("the native I/O error survives")
            .kind(),
        std::io::ErrorKind::PermissionDenied,
        "downcasting preserves the I/O kind"
    );
    let boxed = Error::from_boxed(Box::new(std::io::Error::from(std::io::ErrorKind::NotFound)));
    assert!(
        boxed.downcast_any_ref::<std::io::Error>().is_some(),
        "boxed errors remain inspectable"
    );
    assert!(!format!("{error}").is_empty(), "Display produces a diagnostic");
    assert!(!format!("{error:?}").is_empty(), "Debug produces a diagnostic");

    #[cfg(feature = "gix-lock")]
    {
        let path = std::env::current_dir().map_err(Error::from_error)?.join("resource");
        let _held = gix_lock::Marker::acquire_to_hold_resource(&path, gix_lock::acquire::Fail::Immediately, None, 0)?;
        let error = gix_lock::Marker::acquire_to_hold_resource(&path, gix_lock::acquire::Fail::Immediately, None, 0)
            .expect_err("a second marker conflicts with the existing lock");
        assert!(
            error.is_retryable(),
            "lock contention is recoverable without experimental APIs"
        );
        assert!(
            !error.is_permission_denied(),
            "lock contention is not an authorization failure"
        );
        assert_eq!(
            error
                .downcast_any_ref::<std::io::Error>()
                .expect("lock errors preserve their native cause")
                .kind(),
            std::io::ErrorKind::AlreadyExists,
            "the original lock contention error survives"
        );
    }
    Ok(())
}
