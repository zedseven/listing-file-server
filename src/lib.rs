//! A library that provides a `ListingFileServer` struct that shows directory
//! listings for directory requests.

// Linting rules
#![warn(
	missing_crate_level_docs,
	missing_docs,
	clippy::complexity,
	clippy::correctness,
	clippy::perf,
	clippy::style,
	clippy::suspicious,
	clippy::pedantic,
	clippy::filetype_is_file,
	clippy::str_to_string
)]
#![allow(
	dead_code,
	unused_macros,
	clippy::cast_possible_truncation,
	clippy::cast_possible_wrap,
	clippy::cast_precision_loss,
	clippy::cast_sign_loss,
	clippy::doc_markdown,
	clippy::module_name_repetitions,
	clippy::similar_names,
	clippy::too_many_lines,
	clippy::unnecessary_wraps
)]

// Modules
mod server;

// Exports
pub use self::server::ListingFileServer;
