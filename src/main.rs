extern crate cargo;
extern crate cargo_pack;
extern crate cargo_pack_docker as docker;
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;

use cargo::core::{compiler::CompileMode, shell::Verbosity};
use cargo::ops;
use cargo::util::Config;
use cargo_pack::CargoPack;
use clap::{App, Arg, SubCommand};
use crate::docker::{Docker, PackDockerConfig};

fn doit(
    config: &mut Config,
    package: Option<String>,
    is_release: bool,
    is_no_build: bool,
    tag: Option<&str>,
) {
    config.shell().set_verbosity(Verbosity::Normal);
    debug!("using config {:?}", config);
    let pack = CargoPack::new(&config, package.clone()).expect("initializing cargo-pack failed");
    let package = package.into_iter().collect::<Vec<_>>();
    let package = ops::Packages::Packages(package);
    if !is_no_build {
        // TODO: receive from user via CLI
        let compile_opts = {
            let mut default = ops::CompileOptions::new(&config, CompileMode::Build)
                .expect("makeing compile option failed");
            default.spec = package;
            default.build_config.release = is_release;
            default
        };

        debug!("using compile option {:?}", compile_opts);
        ops::compile(pack.ws(), &compile_opts).expect("build failed");
    }

    let docker_config: PackDockerConfig = pack
        .decode_from_manifest()
        .expect("decoding pack-docker config failed");
    debug!("using docker config {:?}", docker_config);
    let docker = Docker::new(
        docker_config,
        pack,
        tag.into_iter().map(|tag| tag.to_string()).collect(),
        is_release,
    );
    docker.pack().expect("pack failed");
}

fn main() {
    env_logger::init();
    let opts = App::new("cargo")
        .subcommand(
            SubCommand::with_name("pack-docker")
                .version(env!("CARGO_PKG_VERSION"))
                .about(env!("CARGO_PKG_DESCRIPTION"))
                .author("κeen")
                .arg(
                    Arg::with_name("package")
                        .help("parent package to pack")
                        .takes_value(true)
                        .multiple(true)
                        .short("p")
                        .long("package"),
                ).arg(
                    Arg::with_name("release")
                        .help("build with release profile")
                        .long("release"),
                ).arg(
                    Arg::with_name("no-build")
                        .help("do not build rust before packing the docker image")
                        .long("no-build"),
                ).arg(Arg::with_name("TAG").help("tag of the docker image to build")),
        ).get_matches();

    let opts = opts
        .subcommand_matches("pack-docker")
        .expect("cargo-pack-docker must be used as a subcommand");
    let tag = opts.value_of("TAG");
    let packages = opts
        .values_of("package")
        .map(|vs| vs.into_iter().map(|p| p.to_string()).collect::<Vec<_>>());
    let is_release = opts.is_present("release");
    let is_no_build = opts.is_present("no-build");
    debug!(
        "tag: {:?}, package: {:?}, is_release: {:?}",
        tag, packages, is_release
    );
    let mut config = Config::default().expect("failed to create config");
    match packages {
        None => doit(&mut config, None, is_release, is_no_build, tag),
        Some(packages) => for package in packages {
            doit(&mut config, Some(package), is_release, is_no_build, tag);
        },
    }
}
