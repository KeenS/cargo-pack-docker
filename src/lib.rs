extern crate cargo;
extern crate cargo_pack;
extern crate rustc_serialize;
#[macro_use]
extern crate error_chain;
extern crate handlebars;
#[macro_use]
extern crate tojson_macros;
extern crate tempdir;
#[macro_use]
extern crate log;
extern crate semver;

mod docker;
pub use docker::*;

mod error {
    error_chain!{
        foreign_links {
            Io(::std::io::Error);
            Cargo(Box<::cargo::CargoError>);
            CargPack(::cargo_pack::error::Error);
        }
    }
}
