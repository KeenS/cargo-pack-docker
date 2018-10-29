#![deny(dead_code)]
extern crate cargo;
extern crate cargo_pack;
extern crate copy_dir;
extern crate handlebars;
#[macro_use]
extern crate log;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tempdir;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate which;

mod docker;
pub use docker::*;

mod error {
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
