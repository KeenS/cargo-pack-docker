```
$ cargo pack-docker
--bin binname

```


``` toml
[package.metadata.pack]
default-packers = ["docker"]
files = ["README.md"]

[[package.metadata.pack.docker]]
base-image = "ubuntu:16.04"
bin = "aaa"

[[package.metadata.pack.docker]]
base-image = "ubuntu:16.04"
bin = "bbb"

```
