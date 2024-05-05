#![cfg_attr(docsrs, feature(doc_cfg))]

pub use ffi;

#[macro_use]
mod rt;

pub use auto::*;
mod auto;

pub mod functions {
    pub use super::auto::functions::*;
}

pub use rt::*;
