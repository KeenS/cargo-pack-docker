#![deny(dead_code)]

pub mod docker;
pub use crate::docker::*;

mod error {
    use failure::Fail;

    #[derive(Debug, Fail)]
    pub enum Error {
        #[fail(display = "No bins found. Cargo pack-docker only operates on bin crates")]
        NoBins,
        #[fail(display = "ambiguous bin name: {:?}", _0)]
        AmbiguousBinName(Vec<String>),
        #[fail(display = "bin '{}' doesn't exist", _0)]
        BinNotFound(String),
    }
    pub type Result<T> = ::std::result::Result<T, ::failure::Error>;
}
