use std::path::Path;
use cargo_pack::CargoPack;
use error::*;
use std::fs;
use handlebars::{Handlebars, no_escape};
use tempdir::TempDir;
use std::fs::File;
use std::io::{Write, BufWriter};
use std::process::Command;
use semver::Version;
use cargo::util::paths;
use cargo::core::Workspace;


#[derive(RustcDecodable, Debug)]
pub struct PackDocker {
    entrypoint: Option<Vec<String>>,
    cmd: Option<Vec<String>>,
    base_image: String,
    bin: Option<String>,
    inject: Option<String>,
    name: Option<String>,
}

#[derive(RustcDecodable, Debug)]
pub struct PackDockerConfig {
    docker: Vec<PackDocker>,
}

// assuming single bin.
pub struct Docker<'cfg> {
    config: PackDockerConfig,
    pack: CargoPack<'cfg>,
    tags: Vec<String>,
}


#[derive(RustcDecodable, RustcEncodable, ToJson, Debug)]
pub struct DockerfileConfig {
    entrypoint: Option<String>,
    cmd: Option<String>,
    baseimage: String,
    files: Vec<String>,
    bin: String,
    inject: String,
}

impl PackDocker {
    fn base_name(&self, pack: &CargoPack) -> String {
        if let Some(ref name) = self.name {
            // strip right side of `:`
            name.to_string()
        } else {
            pack.package().unwrap().name().to_string()
        }
    }

    fn name(&self, pack: &CargoPack) -> String {
        if let Some(ref name) = self.name {
            name.to_string()
        } else {
            let package = pack.package().unwrap();
            format!("{}{}", package.name(), package.version())
        }
    }
}

impl<'cfg> Docker<'cfg> {
    pub fn new(config: PackDockerConfig, pack: CargoPack<'cfg>, tags: Vec<String>) -> Self {
        Docker {
            config: config,
            pack: pack,
            tags: tags,
        }
    }

    pub fn pack(&self) -> Result<()> {
        debug!("tags: {:?}, config: {:?}", self.tags, self.config);
        debug!("workspace: {:?}", self.pack.package());
        debug!("preparing");
        for pack_docker in self.targets() {
            let tmpdir = self.prepare(pack_docker)?;
            debug!("building a image");
            self.build(tmpdir)?;

        }
        Ok(())
    }

    fn prepare(&self, pack_docker: &PackDocker) -> Result<TempDir> {
        let tmp = TempDir::new("cargo-pack-docker")?;
        debug!("created: {:?}", tmp);
        self.copy_files(&tmp)?;
        let bin = self.add_bin(&tmp, pack_docker)?;
        let data = DockerfileConfig {
            entrypoint: pack_docker.entrypoint
                .as_ref()
                .map(|e| e.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", ")),
            cmd: pack_docker.cmd
                .as_ref()
                .map(|c| c.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", ")),
            baseimage: pack_docker.base_image.clone(),
            files: self.pack.files().into(),
            bin: bin,
            inject: pack_docker.inject.as_ref().map(|s| s.as_ref()).unwrap_or("").to_string(),
        };
        self.gen_dockerfile(&tmp, &data)?;
        Ok(tmp)
    }

    fn build<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let image_name = self.app_name()?;
        let image_version = self.app_version()?;
        let status = Command::new("/usr/bin/docker").current_dir(&path)
            .arg("build")
            .arg(path.as_ref().to_str().unwrap())
            .args(&["-t", format!("{}:{}", image_name, image_version).as_str()])
            .spawn()?
            .wait()?;

        if status.success() {
            Ok(())
        } else {
            Err("docker command faild".into())
        }
    }

    fn copy_files<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        for file in self.pack.files() {
            let to = path.as_ref().join(file);
            debug!("copying file: from {:?} to {:?}", file, to);
            fs::copy(file, to)?;
        }
        Ok(())
    }

    fn add_bin<P: AsRef<Path>>(&self, path: P, pack_docker: &PackDocker) -> Result<String> {
        let bins = self.pack
            .package()?
            .targets()
            .iter()
            .filter(|t| t.is_bin())
            .collect::<Vec<_>>();
        let primary_bin_name = if 0 < bins.len() {
            bins[0].name()
        } else {
            return Err("no bins found".into());
        };

        let name = pack_docker.bin.as_ref().map(|s| s.as_ref()).unwrap_or(primary_bin_name);
        let from = self.pack
            .ws()
            .target_dir()
            .join("release")
            .open_ro(name, self.pack.ws().config(), "waiting for the bin")?;
        let from = from.path();
        let to = path.as_ref().join(name);
        debug!("copying file: from {:?} to {:?}", from, to);
        fs::copy(from, to)?;
        Ok(name.to_string())
    }

    fn app_name(&self) -> Result<&str> {
        Ok(self.pack.package()?.name())
    }

    fn app_version(&self) -> Result<&Version> {
        Ok(self.pack.package()?.version())
    }

    fn targets(&self) -> Vec<&PackDocker> {
        if self.tags.len() == 0 {
            self.config.docker.iter().collect()
        } else {
            // TODO: warn non existing tags
            self.config
                .docker
                .iter()
                .filter(|p| self.tags.contains(&p.base_name(&self.pack)))
                .collect()
        }
    }

    fn gen_dockerfile<P: AsRef<Path>>(&self, path: P, data: &DockerfileConfig) -> Result<()> {
        let dockerfile = path.as_ref().join("Dockerfile");
        debug!("generating {:?}", dockerfile);
        let file = File::create(dockerfile)?;
        debug!("file create succeeded. templating");
        let mut buf = BufWriter::new(file);
        let template = r#"
FROM {{ baseimage }}

RUN mkdir -p /opt/app/bin
{{#each files as |file| ~}}
  COPY {{ file }} /opt/app
{{/each~}}
COPY {{bin}} /opt/app/bin
WORKDIR /op/app

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
        handlebars.register_template_string("dockerfile", template)
            .expect("internal error: illegal template");

        handlebars.renderw("dockerfile", data, &mut buf).unwrap();
        debug!("templating done");
        let _ = buf.flush()?;
        debug!("content:{}",
               paths::read(path.as_ref().join("Dockerfile").as_ref())?);

        Ok(())
    }
}
// mktmpdir
// cp files to tmpdir
// output Dockerfile
// docker build -f Dockerfile ./
