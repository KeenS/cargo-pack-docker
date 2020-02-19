use crate::error::*;
use cargo_pack::CargoPack;
use copy_dir;
use failure::format_err;
use handlebars::{no_escape, Handlebars};
use log::debug;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::process::Command;
use tempdir::TempDir;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct PackDocker {
    entrypoint: Option<Vec<String>>,
    cmd: Option<Vec<String>>,
    base_image: String,
    bin: Option<String>,
    inject: Option<String>,
    tag: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct PackDockerConfig {
    docker: Vec<PackDocker>,
}

// assuming single bin.
pub struct Docker {
    config: PackDockerConfig,
    pack: CargoPack,
    tags: Vec<String>,
    is_release: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DockerfileConfig {
    entrypoint: Option<String>,
    cmd: Option<String>,
    baseimage: String,
    files: Vec<String>,
    bin: String,
    inject: String,
}

impl PackDocker {
    fn base_name(&self, docker: &Docker) -> Result<String> {
        self.tag(docker).map(|name| {
            name.rsplitn(2, ':')
                .last()
                // should be safe but not confident
                .unwrap()
                .to_string()
        })
    }

    fn bin_name<'a>(&'a self, docker: &'a Docker) -> Result<&'a str> {
        let bins = docker
            .pack
            .package()?
            .targets
            .iter()
            .filter(|t| t.kind.contains(&"bin".to_string()))
            .map(|t| &t.name)
            .collect::<Vec<_>>();

        if let Some(name) = self.bin.as_ref() {
            if bins.contains(&name) {
                return Ok(name);
            } else {
                return Err(Error::BinNotFound(name.clone()).into());
            }
        }
        match bins.len() {
            0 => Err(Error::NoBins.into()),
            1 => Ok(bins.get(0).unwrap()),
            _ => Err(Error::AmbiguousBinName(bins.into_iter().map(Into::into).collect()).into()),
        }
    }

    fn tag(&self, docker: &Docker) -> Result<String> {
        if let Some(ref tag) = self.tag {
            Ok(tag.to_string())
        } else {
            let bin_name = self.bin_name(docker)?;
            let package = docker.pack.package().unwrap();
            let version = if docker.is_release {
                package.version.to_string()
            } else {
                "latest".to_string()
            };
            Ok(format!("{}:{}", bin_name, version))
        }
    }
}

impl<'cfg> Docker {
    pub fn new(
        config: PackDockerConfig,
        pack: CargoPack,
        tags: Vec<String>,
        is_release: bool,
    ) -> Self {
        Docker {
            config,
            pack,
            tags,
            is_release,
        }
    }

    pub fn pack(&self) -> Result<()> {
        debug!("tags: {:?}, config: {:?}", self.tags, self.config);
        debug!("workspace: {:?}", self.pack.package());
        debug!("preparing");
        for pack_docker in self.targets() {
            let tmpdir = self.prepare(pack_docker)?;
            debug!("building a image");
            self.build(tmpdir, pack_docker)?;
        }
        Ok(())
    }

    fn prepare(&self, pack_docker: &PackDocker) -> Result<TempDir> {
        let tmp = TempDir::new("cargo-pack-docker")?;
        debug!("created: {:?}", tmp);
        self.copy_files(&tmp)?;
        let bin = self.add_bin(&tmp, pack_docker)?;
        let data = DockerfileConfig {
            entrypoint: pack_docker.entrypoint.as_ref().map(|e| {
                e.iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(", ")
            }),
            cmd: pack_docker.cmd.as_ref().map(|c| {
                c.iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(", ")
            }),
            baseimage: pack_docker.base_image.clone(),
            files: self.pack.files().into(),
            bin: bin,
            inject: pack_docker
                .inject
                .as_ref()
                .map(|s| s.as_ref())
                .unwrap_or("")
                .to_string(),
        };
        self.gen_dockerfile(&tmp, &data)?;
        Ok(tmp)
    }

    fn build<P: AsRef<Path>>(&self, path: P, pack_docker: &PackDocker) -> Result<()> {
        let image_tag = pack_docker.tag(self)?;
        // FIXME: take from user
        let dockerbin = ::which::which("docker")?;
        let status = Command::new(dockerbin)
            .current_dir(&path)
            .arg("build")
            .arg(path.as_ref().to_str().unwrap())
            .args(&["-t", image_tag.as_str()])
            .spawn()?
            .wait()?;

        if status.success() {
            Ok(())
        } else {
            Err(format_err!("docker command faild"))
        }
    }

    fn copy_files<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        for file in self.pack.files() {
            let to = path.as_ref().join(file);
            debug!("copying file: from {:?} to {:?}", file, to);
            copy_dir::copy_dir(file, to)?;
        }
        Ok(())
    }

    fn add_bin<P: AsRef<Path>>(&self, path: P, pack_docker: &PackDocker) -> Result<String> {
        let name = pack_docker.bin_name(self)?;
        let from = if self.is_release {
            self.pack
                .metadata()
                .target_directory
                .join("release")
                .join(&name)
        } else {
            self.pack
                .metadata()
                .target_directory
                .join("debug")
                .join(&name)
        };

        let to = path.as_ref().join(&name);
        debug!("copying file: from {:?} to {:?}", from, to);
        fs::copy(from, to)?;
        Ok(name.into())
    }

    fn targets(&self) -> Vec<&PackDocker> {
        if self.tags.len() == 0 {
            self.config.docker.iter().collect()
        } else {
            // TODO: warn non existing tags
            self.config
                .docker
                .iter()
                .filter(|p| {
                    p.base_name(&self)
                        .map(|name| self.tags.contains(&name))
                        .unwrap_or(false)
                })
                .collect()
        }
    }

    fn gen_dockerfile<P: AsRef<Path>>(&self, path: P, data: &DockerfileConfig) -> Result<()> {
        let dockerfile = path.as_ref().join("Dockerfile");
        debug!("generating {:?}", dockerfile);
        let file = File::create(dockerfile)?;
        debug!("Dockerfile creation succeeded.");
        debug!("templating with {:?}", data);
        let mut buf = BufWriter::new(file);
        let template = r#"
FROM {{ baseimage }}

RUN mkdir -p /opt/app/bin
{{#each files as |file| ~}}
  COPY {{ file }} /opt/app
{{/each~}}
COPY {{bin}} /opt/app/bin
WORKDIR /opt/app

{{inject}}

{{#if entrypoint ~}}
ENTRYPOINT [{{entrypoint}}]
{{else ~}}
ENTRYPOINT ["/opt/app/bin/{{bin}}"]
{{/if ~}}
{{#if cmd ~}}
CMD [{{cmd}}]
{{/if}}
"#;
        let mut handlebars = Handlebars::new();

        handlebars.register_escape_fn(no_escape);
        handlebars
            .register_template_string("dockerfile", template)
            .expect("internal error: illegal template");

        handlebars
            .render_to_write("dockerfile", data, &mut buf)
            .unwrap();
        debug!("templating done");
        let _ = buf.flush()?;
        debug!(
            "content:{}",
            fs::read_to_string(path.as_ref().join("Dockerfile"))?
        );

        Ok(())
    }
}
// mktmpdir
// cp files to tmpdir
// output Dockerfile
// docker build -f Dockerfile ./
