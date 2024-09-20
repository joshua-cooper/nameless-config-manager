use globset::GlobSet;
use serde::Deserialize;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Deserialize)]
struct Config {
    sources: GlobSet,
    target: String,
    reload: Option<String>,
}

pub fn foo<P: AsRef<Path>>(dir: P) {
    let config_file_name = OsStr::new("build.toml");
    let mut config_list = Vec::<(PathBuf, Config)>::new();

    for result in WalkDir::new(dir) {
        let entry = result.unwrap();
        let path = entry.path();

        while config_list
            .last()
            .is_some_and(|(parent, _)| !path.starts_with(parent))
        {
            config_list.pop();
        }

        if path.is_dir() {
            let config_path = path.join(config_file_name);

            if config_path.exists() {
                let config = fs::read_to_string(config_path).unwrap();
                let config = toml::from_str(&config).unwrap();
                config_list.push((path.to_owned(), config));
            }

            continue;
        }

        if path.file_name() == Some(config_file_name) {
            continue;
        }

        for (root, config) in &config_list {
            let relative_path = path
                .strip_prefix(root)
                .expect("all config roots are parents to path");

            if config.sources.is_match(&relative_path) {
                println!("({}) {}", root.display(), relative_path.display());
                // TODO:
                //   - Copy files to build dir
                //   - If template, expand before copying
                //   - Append build path : target path mapping to activation builder
                //   - Append reload to activation builder (if changed?)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_deserialize_config() {
        let toml = r#"
            sources = ["*.toml"]
            target = "$XDG_CONFIG_HOME/foo"
        "#;

        toml::from_str::<Config>(toml).unwrap();
    }
}
