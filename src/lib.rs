#![deny(dead_code)]
extern crate cargo;
extern crate cargo_pack;
extern crate copy_dir;
#[macro_use]
extern crate error_chain;
extern crate handlebars;
#[macro_use]
extern crate log;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tempdir;

mod docker;
pub use docker::*;

mod error {
    error_chain!{
        foreign_links {
            Io(::std::io::Error);
            Cargo(::cargo::CargoError);
            CargPack(::cargo_pack::error::Error);
        }
        errors {
            NoBins {
                description("no bins found")
                    display("No bins found. Cargo pack-docker only operates on bin crates")
            }
            AmbiguousBinName(names: Vec<String>) {
                description("more than one bins found")
                    display("ambiguous bin name: {:?}", names)
            }
            BinNotFound(name: String) {
                description("specified name doesn't exist")
                    display("bin '{}' doesn't exist", name)
            }
        }
    }
}
