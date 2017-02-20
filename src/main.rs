extern crate rustc_serialize;
extern crate cargo_pack;
extern crate cargo;
extern crate cargo_pack_docker as docker;
extern crate env_logger;


use cargo::util::Config;
use cargo_pack::CargoPack;
use docker::{Docker, PackDockerConfig};

fn main() {
    env_logger::init().expect("failed to init env logger");
    let config = Config::default().expect("config");
    let pack = CargoPack::new(&config).expect("failed");
    let docker: PackDockerConfig = pack.decode_from_manifest().expect("failed");
    let docker = Docker::new(docker, pack);
    docker.pack().expect("pack failed");
}
