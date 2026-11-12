//! Loads an xD configuration from Nickel. The configuration is checked
//! twice: by the contracts in `nickel/xd.ncl` while Nickel evaluates it, and
//! by [`Scheme::validate`] once it is a Rust value.

use std::path::{Path, PathBuf};

use nickel_embed::{Program, Source};
use serde::Deserialize;
use thiserror::Error;
use xd_core::{Scheme, SchemeError};

/// The parts of a system's configuration the core understands so far.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct System {
    pub scheme: Scheme,
}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("{0}")]
    Nickel(#[from] nickel_embed::Error),
    #[error("the configuration doesn't have the shape xD expects: {0}")]
    Shape(String),
    #[error(transparent)]
    Scheme(#[from] SchemeError),
}

/// The folder holding `xd.ncl` and the built-in configurations such as
/// `jd.ncl`, as shipped in this repository.
pub fn library_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../nickel")
}

/// Evaluates `path`, with [`library_dir`] on the import path so a
/// configuration can `import "xd.ncl"` or `import "jd.ncl"`.
pub fn load(path: impl AsRef<Path>) -> Result<System, LoadError> {
    let mut program = Program::builder()
        .source(Source::file(path.as_ref()))
        .import_path(library_dir())
        .build()?;
    let system: System = program
        .eval_deep_for_export()?
        .to_serde()
        .map_err(|e| LoadError::Shape(e.to_string()))?;
    system.scheme.validate()?;
    Ok(system)
}
