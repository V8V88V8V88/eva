#![warn(clippy::all, clippy::pedantic)]

pub mod prelude {
    pub use bucky_common::{Content, Input, RequestError, Response, ResponseParseError, ToGmi};
}

#[cfg(feature = "data")]
pub use bucky_data as data;

#[cfg(feature = "file")]
pub use bucky_file as file;

#[cfg(feature = "finger")]
pub use bucky_finger as finger;

#[cfg(feature = "gemini")]
pub use bucky_gemini as gemini;

#[cfg(feature = "gopher")]
pub use bucky_gopher as gopher;

#[cfg(feature = "spartan")]
pub use bucky_spartan as spartan;
