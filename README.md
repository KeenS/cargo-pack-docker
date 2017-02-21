```
$ cargo pack-docker
cargo pack-docker [-p package] TAG
```


``` toml
[package.metadata.pack]
default-packers = ["docker"]
files = ["README.md"]

[[package.metadata.pack.docker]]
tag = "hoge:0.1.0"
base-image = "ubuntu:16.04"
bin = "aaa"
entrypoint = ["aa", "bb"]
cmd = ["c", "d"]
inject = "
ENV RUST_LOG debug
RUN apt install libpq-dev
"

[[package.metadata.pack.docker]]
base-image = "ubuntu:16.04"
bin = "bbb"

```
