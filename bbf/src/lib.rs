pub mod builder;
pub mod ffi;
pub mod format;
pub mod reader;

#[cfg(feature = "uniffi-bindings")]
pub mod bindings;
#[cfg(feature = "uniffi-bindings")]
use bindings::{BbfBuilder, BbfError, BbfReader, MediaType};

pub use builder::BBFBuilder;
pub use format::BBFMediaType;
pub use reader::BBFReader;

#[cfg(feature = "uniffi-bindings")]
uniffi::include_scaffolding!("bbf");
