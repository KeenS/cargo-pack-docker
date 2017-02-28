# cargo-pack-docker
A [`cargo-pack`](https://github.com/KeenS/cargo-pack)er for docker; package your application into a docker image to deploy without Dockerfile

THIS PRODUCT IS ALPHA STATUS. USE AT YOUR OWN RISK
# install

```
cargo install cargo-pack-docker
```

# Usage

```
cargo pack-docker [-p package] [--release] [TAG]
# if your configurated tag in Cargo.toml is hoge:0.1.0, the TAG will be hoge
# if TAG is omitted and you have only one `[[package.metadata.pack.docker]]` section, it will be used
```

# Configulation


``` toml
# configuration of cargo-pack
[package.metadata.pack]
default-packers = ["docker"]
# files will be placet to /opt/app
files = ["README.md"]

# configuration of cargo-pack-docker
[[package.metadata.pack.docker]]
# tag of the created image. Can be omitted.
# Default to PACKAGE_NAME:latest for debug profile
# and PACKAGE_NAME:PACKAGE_VERSION for release profile
tag = "hoge:0.1.0"
# base image of the docker image. Required.
base-image = "ubuntu:16.04"
# the bin to include into the docker image.
# will be placed to /opt/app/bin/
# can be omitted if the project have only one binary target.
bin = "aaa"
# `ENTRYPOINT` of Dockerfile. optional.
entrypoint = ["aa", "bb"]
# `CMD` of Dockerfile. optional.
cmd = ["c", "d"]
# inject command into the Dockerfile. optional
inject = "
ENV RUST_LOG debug
RUN apt install libpq-dev
"

# you can write another configuration 
[[package.metadata.pack.docker]]
base-image = "ubuntu:16.04"
bin = "bbb"
```

with the first configuration, build a docker image with this Dockerfile content:

```
FROM ubuntu:16.04

RUN mkdir -p /opt/app/bin
COPY README.md /opt/app
COPY aaa /opt/app/bin
WORKDIR /opt/app

ENV RUST_LOG debug
RUN apt install libpq-dev


ENTRYPOINT ["aa", "bb"]
CMD ["c", "d"]

```
