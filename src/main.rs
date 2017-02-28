extern crate rustc_serialize;
extern crate cargo_pack;
extern crate cargo;
extern crate cargo_pack_docker as docker;
extern crate env_logger;
extern crate clap;


use cargo::util::Config;
use cargo_pack::CargoPack;
use cargo::ops;
use docker::{Docker, PackDockerConfig};
use clap::{App, Arg, SubCommand};

fn main() {
    env_logger::init().expect("failed to init env logger");
    let opts = App::new("cargo")
        .subcommand(SubCommand::with_name("pack-docker")
            .version(env!("CARGO_PKG_VERSION"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .author("κeen")
            .arg(Arg::with_name("package")
                .help("parent package to pack")
                .takes_value(true)
                .short("p")
                .long("package"))
            .arg(Arg::with_name("release")
                .help("build with release profile")
                .long("release"))
            .arg(Arg::with_name("TAG").help("tag of the docker image to build")))
        .get_matches();

    let tag = opts.value_of("TAG");
    let package = opts.value_of("package");
    let is_release = opts.is_present("release");
    let config = Config::default().expect("config");
    let pack = CargoPack::new(&config, package.map(|s| s.to_string())).expect("failed");
    config.configure(0, None, &None, false, false).expect("reconfigure failed");
    let packages = package.iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>();
    let packages = ops::Packages::Packages(packages.as_ref());
    // TODO: receive from user via CLI
    let compile_opts = ops::CompileOptions {
        config: &config,
        jobs: None,
        target: None,
        features: &vec![],
        all_features: false,
        no_default_features: false,
        spec: packages,
        release: is_release,
        mode: ops::CompileMode::Build,
        filter: ops::CompileFilter::Everything,
        message_format: ops::MessageFormat::Human,
        target_rustdoc_args: None,
        target_rustc_args: None,
    };
    ops::compile(pack.ws(), &compile_opts).expect("build failed");

    let docker: PackDockerConfig = pack.decode_from_manifest().expect("failed");
    let docker = Docker::new(docker,
                             pack,
                             tag.into_iter().map(|tag| tag.to_string()).collect(),
                             is_release);
    docker.pack().expect("pack failed");
}
