use cargo_pack::CargoPack;
use cargo_pack_docker::{Docker, PackDockerConfig};
use clap::{App, Arg, SubCommand};
use log::debug;
use std::process::Command;

fn doit(package: Option<String>, is_release: bool, is_no_build: bool, tag: Option<&str>) {
    let pack = CargoPack::new(package.clone()).expect("initializing cargo-pack failed");
    let package = package.into_iter().collect::<Vec<_>>();
    if !is_no_build {
        let build_mode = if is_release { "--release" } else { "--debug" };
        let mut build_options = package
            .iter()
            .flat_map(|name| vec!["--package", name])
            .collect::<Vec<_>>();
        build_options.push(build_mode);
        let mut args = vec!["build"];
        args.append(&mut build_options);
        let mut cmd = Command::new("cargo");
        let cmd = cmd.args(&args);
        debug!("running command {:?}", cmd);
        match cmd.status() {
            Ok(_) => (),
            Err(e) => {
                eprintln!("failed to build: {}", e);
                return;
            }
        }
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
                )
                .arg(
                    Arg::with_name("release")
                        .help("build with release profile")
                        .long("release"),
                )
                .arg(
                    Arg::with_name("no-build")
                        .help("do not build rust before packing the docker image")
                        .long("no-build"),
                )
                .arg(Arg::with_name("TAG").help("tag of the docker image to build")),
        )
        .get_matches();

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
    match packages {
        None => doit(None, is_release, is_no_build, tag),
        Some(packages) => {
            for package in packages {
                doit(Some(package), is_release, is_no_build, tag);
            }
        }
    }
}
