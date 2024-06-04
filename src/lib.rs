//! Free interview 2024

// Macro options
#![recursion_limit = "512"]
// Lints
#![warn(unsafe_code)]
#![deny(unused_results)]
#![warn(missing_docs)]
// Clippy lint options
// see clippy.toml
// https://rust-lang.github.io/rust-clippy/master/index.html
#![deny(
    // Pedantic
    clippy::pedantic,
)]
#![warn(
    // Restriction
    clippy::allow_attributes_without_reason,
    clippy::decimal_literal_representation,
    clippy::clone_on_ref_ptr,
    clippy::create_dir,
    clippy::dbg_macro,
    clippy::default_union_representation,
    clippy::exit,
    clippy::fn_to_numeric_cast_any,
    clippy::get_unwrap,
    clippy::if_then_some_else_none,
    clippy::let_underscore_must_use,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::mod_module_files,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::same_name_method,
    clippy::separated_literal_suffix,
    clippy::shadow_unrelated,
    clippy::try_err,
    clippy::undocumented_unsafe_blocks,
    clippy::unneeded_field_pattern,
    clippy::unseparated_literal_suffix,
    clippy::verbose_file_reads,
    clippy::empty_drop,
    clippy::mixed_read_write_in_expression,
    // clippy::pub_use,

    // Nursery
    clippy::cognitive_complexity,
    clippy::debug_assert_with_mut_call,
    clippy::future_not_send,
    clippy::imprecise_flops,

    // Cargo
//     clippy::multiple_crate_versions, // check from time to time
    clippy::wildcard_dependencies,
)]
#![allow(clippy::match_bool)]

/// Error module
mod error;

/// Lambda app module
mod lambda_app;

/// http Pagination
mod pagination;

/// Sandboxing
mod sandbox;

pub use error::HttpErr;
pub use lambda_app::{BashApp, LambdaAppKind as LambdaApp, Trait as LambdaTrait};
pub use pagination::Pagination;
pub use sandbox::{
    BubbleWrap as SandboxBubbleWrap, Host as SandboxHost, SandboxKind as Sandbox,
    Trait as SandboxTrait,
};
