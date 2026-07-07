//! Concrete provider adapters (Spec 116 §10).
//!
//! Each adapter implements `ProviderAdapter` from `crate::work_item::adapter`.
//! Adding a new provider = one new file in this directory, plus a
//! `register()` call in `bd` / `bd-cli`-style installers.

pub mod bd;
pub mod none;

pub use bd::BdAdapter;
pub use none::NoneAdapter;
