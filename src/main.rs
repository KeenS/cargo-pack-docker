extern crate rustc_serialize;
extern crate cargo_pack;
extern crate cargo;
extern crate cargo_pack_docker as docker;
extern crate env_logger;
extern crate clap;


use cargo::util::Config;
use cargo_pack::CargoPack;
use docker::{Docker, PackDockerConfig};
use clap::{App, Arg};

fn main() {
    env_logger::init().expect("failed to init env logger");
    let opts = App::new("cargo-pack-docker")
        .version("0.1.0")
        .about("pack artifacts into a docker image")
        .author("κeen")
        .arg(Arg::with_name("package")
            .help("parent package to pack")
            .takes_value(true)
            .short("p")
            .long("package"))
        .arg(Arg::with_name("TAG").help("tag of the docker image to build"))
        .get_matches();

    let tag = opts.value_of("TAG");
    let package = opts.value_of("package");

    let config = Config::default().expect("config");
    let pack = CargoPack::new(&config, package.map(|s| s.to_string())).expect("failed");
    let docker: PackDockerConfig = pack.decode_from_manifest().expect("failed");
    let docker = Docker::new(docker,
                             pack,
                             tag.into_iter().map(|tag| tag.to_string()).collect());
    docker.pack().expect("pack failed");
}
