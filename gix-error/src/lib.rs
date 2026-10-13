//! Type-erased errors for stable plumbing APIs.
//!
//! # Stability
//!
//! The supported API consists of [`Error`], [`Result`], native-error construction and inspection,
//! and [`Error::is_retryable()`] and [`Error::is_permission_denied()`] for lock recovery.
//! It preserves concrete error types and native sources for downcasting and standard error interoperability.
//! The `error-print-location` feature is also supported.
//!
//! All other APIs require the non-default `unstable` feature, including exceptions, extension traits,
//! message construction, classification details, metadata, and test helpers. Pin an exact crate version
//! when using them. The `bstr`, `anyhow`, `auto-chain-error`, and `tree-error` features enable `unstable`.
//!
//! ```
//! use gix_error::{Error, Result};
//!
//! fn read(path: &std::path::Path) -> Result<Vec<u8>> {
//!     std::fs::read(path).map_err(Error::from_error)
//! }
//! ```
#![cfg_attr(feature = "unstable", doc = include_str!("unstable.md"))]
#![cfg_attr(
    all(doc, feature = "document-features"),
    doc = ::document_features::document_features!()
)]
#![deny(missing_docs, unsafe_code)]
#[cfg(feature = "unstable")]
pub mod exn;
#[cfg(not(feature = "unstable"))]
#[allow(dead_code)]
pub(crate) mod exn;
#[cfg(feature = "unstable")]
pub mod types;
#[cfg(not(feature = "unstable"))]
#[allow(dead_code, unused_imports)]
pub(crate) mod types;

#[cfg(feature = "bstr")]
#[gix_macros::unstable]
pub use bstr;
#[gix_macros::unstable]
pub use exn::{
    ext::{BoxedResultExt, ErrorExt, OptionExt, ResultExt},
    impls::Exn,
};

/// A type-erased error preserving concrete payloads, native sources, and caller locations.
///
/// [`Self::from_error()`] and [`Self::from_boxed()`] preserve the original error's native source chain.
/// [`Self::iter_errors()`] and [`Self::downcast_any_ref()`] inspect concrete types throughout the error,
/// including nested errors and sources. Standard [`source()`](std::error::Error::source) traversal
/// follows one causal path; use [`Self::iter_errors()`] to inspect all stored errors.
///
/// This type implements [`std::error::Error`], [`Send`], and [`Sync`]. Its representation is private.
/// Normal [`Display`](std::fmt::Display) shows the root diagnostic; [`Debug`](std::fmt::Debug) reports
/// captured error context. Caller-location printing requires `error-print-location`.
pub struct Error {
    #[cfg(any(feature = "tree-error", not(feature = "auto-chain-error")))]
    inner: error::Inner,
    #[cfg(all(feature = "auto-chain-error", not(feature = "tree-error")))]
    inner: types::ChainedError,
}

fn root_error_eq(mut error: &(dyn std::error::Error + 'static), other: &str) -> bool {
    while let Some(nested) = error.downcast_ref::<Error>() {
        error = nested.error();
    }
    error.to_string() == other
}

#[gix_macros::unstable]
impl PartialEq<str> for Error {
    fn eq(&self, other: &str) -> bool {
        root_error_eq(self.error(), other)
    }
}

#[gix_macros::unstable]
impl PartialEq<&str> for Error {
    fn eq(&self, other: &&str) -> bool {
        <Self as PartialEq<str>>::eq(self, other)
    }
}

#[gix_macros::unstable]
impl PartialEq<String> for Error {
    fn eq(&self, other: &String) -> bool {
        <Self as PartialEq<str>>::eq(self, other)
    }
}

/// A result with a type-erased [`Error`], defaulting to unit success.
pub type Result<T = ()> = std::result::Result<T, Error>;

/// A result with an [`Exn<E>`](Exn) error, defaulting to unit success and an erased error type.
///
/// `ExnResult<T>` uses the same [`exn::Untyped`] marker as bare [`Exn`]. Specify `E` to retain a
/// concrete error type; [`ExnMessageResult`] is the shorthand for message contexts. Use these aliases when callers
/// need typed exceptions or exception-tree operations; other public and private code uses [`Result`].
/// All standard result operations and [`ResultExt`] methods remain available. Use [`ResultExt::or_erased()`] for
/// callbacks requiring erased exceptions, and `?` to propagate exceptions into [`Error`].
/// Preserve this alias in public signatures that already expose a concrete error type.
///
/// ```
/// use gix_error::{ErrorExt, ExnResult};
///
/// fn parse_count(input: &str) -> ExnResult<u64, std::num::ParseIntError> {
///     input.parse::<u64>().map_err(ErrorExt::raise_typed)
/// }
///
/// let error = parse_count("not a count").expect_err("the count contains letters");
/// assert_eq!(
///     error.error().kind(),
///     &std::num::IntErrorKind::InvalidDigit,
///     "callers can inspect the concrete error without downcasting"
/// );
/// ```
#[gix_macros::unstable]
pub type ExnResult<T = (), E = exn::Untyped> = std::result::Result<T, Exn<E>>;

/// A result with a [`Message`] exception, defaulting to unit success.
///
/// This is [`ExnResult<T, Message>`](ExnResult). Use it when callers need direct access to the [`Message`]
/// context or exception-tree operations. Other message-based public and private APIs use [`Result<T>`](Result).
/// Construct it with [`ResultExt::or_raise_typed()`], [`ErrorExt::raise_typed()`], or [`bail!`].
///
/// ```
/// use gix_error::{bail, ExnMessageResult};
///
/// fn validate(ready: bool) -> ExnMessageResult {
///     if !ready {
///         bail!("not ready");
///     }
///     Ok(())
/// }
///
/// validate(true)?;
/// assert_eq!(validate(false).expect_err("not ready").error().message, "not ready");
/// # Ok::<(), gix_error::Error>(())
/// ```
#[gix_macros::unstable]
pub type ExnMessageResult<T = ()> = ExnResult<T, Message>;

mod test;
#[gix_macros::unstable]
pub use test::{TestError, TestResult};

#[cfg_attr(not(feature = "unstable"), allow(dead_code))]
mod error;
#[gix_macros::unstable]
pub use error::{Class, classify};

/// Various kinds of concrete errors that implement [`std::error::Error`].
#[cfg_attr(not(feature = "unstable"), allow(dead_code))]
mod concrete;

#[gix_macros::unstable]
pub use concrete::classify::{ClassificationMarker, ResourceExhaustionKind, tag};
#[gix_macros::unstable]
pub use concrete::message::message;
#[gix_macros::unstable]
pub use concrete::metadata::{
    Message, Metadata, MetadataValue, allocation_failure, allocation_limit, cancelled, conflict, corruption, not_found,
    permission_denied, resource_exhaustion, retryable, unauthenticated, unsupported, validation,
};

mod location;
pub(crate) use location::write as write_location;
