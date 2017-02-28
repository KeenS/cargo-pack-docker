# cargo-pack-docker
A [`cargo-pack`](https://github.com/KeenS/cargo-pack)er for docker; package your application into a docker image to deploy without Dockerfile

# Usage

```
cargo pack-docker [-p package] [--release] TAG
# if your configurated tag in Cargo.toml is hoge:0.1.0, the TAG will be hoge
```

# Configulation


``` toml
# configuration of cargo-pack
[package.metadata.pack]
default-packers = ["docker"]
files = ["README.md"]

# configuration of cargo-pack-docker
[[package.metadata.pack.docker]]
# tag of the created image.
# Default to PACKAGE_NAME:latest for debug profile
# and PACKAGE_NAME:PACKAGE_VERSION for release profile
tag = "hoge:0.1.0"
# base image of the docker image
base-image = "ubuntu:16.04"
# the bin to include into the docker image.
# will be placed to /opt/app/bin/
bin = "aaa"
# `ENTRYPOINT` of Dockerfile
entrypoint = ["aa", "bb"]
# `CMD` of Dockerfile
cmd = ["c", "d"]
# inject command into the Dockerfile
inject = "
ENV RUST_LOG debug
RUN apt install libpq-dev
"

# you can write another configuration 
[[package.metadata.pack.docker]]
base-image = "ubuntu:16.04"
bin = "bbb"

```
