//! Environment Mapping
#![cfg_attr(feature = "cargo-clippy", allow(use_self))]
#![feature(try_from)]
#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate getset;

mod error;

pub use error::{Error, ErrorKind, Result};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::{env, fmt};

/// Suffix for environment variables file name.
const ENV_SUFFIX: &str = ".env";

/// Environment Kinds
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Kind {
    /// `Common` or shared environment variables.
    Common,
    /// `Development` specific environment variables.
    Development,
    /// `Test` specific environment variables.
    Test,
    /// `Integration` specific environment variables.
    Integration,
    /// `Staging` specific environment variables.
    Staging,
    /// `Production` specific environment variables.
    Production,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let kind_str: String = (*self).into();
        write!(f, "{}", kind_str)
    }
}

impl<'a> TryFrom<&'a str> for Kind {
    type Error = Error;

    fn try_from(name: &'a str) -> Result<Kind> {
        let kind = match name {
            "common" => Kind::Common,
            "dev" => Kind::Development,
            "test" => Kind::Test,
            "int" => Kind::Integration,
            "stage" => Kind::Staging,
            "prod" => Kind::Production,
            _ => return Err(ErrorKind::InvalidKind(name.to_string()).into()),
        };
        Ok(kind)
    }
}

impl<'a> From<Kind> for &'a str {
    fn from(kind: Kind) -> &'a str {
        match kind {
            Kind::Common => "common",
            Kind::Development => "dev",
            Kind::Test => "test",
            Kind::Integration => "int",
            Kind::Staging => "stage",
            Kind::Production => "prod",
        }
    }
}

impl From<Kind> for String {
    fn from(kind: Kind) -> String {
        String::from(match kind {
            Kind::Common => "common",
            Kind::Development => "dev",
            Kind::Test => "test",
            Kind::Integration => "int",
            Kind::Staging => "stage",
            Kind::Production => "prod",
        })
    }
}

/// A `Config` used when loading environment properties.
#[derive(Builder, Clone, Debug, Eq, Getters, PartialEq, Setters)]
#[builder(setter(into))]
pub struct Config {
    /// The environment `Kind` we are loading.
    #[get = "pub"]
    #[set = "pub"]
    #[builder(default = "self.default_kind()")]
    kind: Kind,
    /// The application name.
    #[get = "pub"]
    #[set = "pub"]
    app_name: String,
    /// Should we read from a `common.env` file?
    #[get = "pub"]
    #[set = "pub"]
    #[builder(default = "false")]
    common: bool,
    /// Does the property file have comments?
    #[get = "pub"]
    #[set = "pub"]
    #[builder(default = "false")]
    comments: bool,
    /// The comment character.
    #[get = "pub"]
    #[set = "pub"]
    #[builder(default = "'#'")]
    comment_char: char,
    /// Should we pull the OS environment into our props?
    #[get = "pub"]
    #[set = "pub"]
    #[builder(default = "false")]
    os: bool,
}

impl ConfigBuilder {
    /// Setup the default `Kind` for a `Config`.
    fn default_kind(&self) -> Kind {
        Kind::Development
    }
}

/// The `Environment` of the given kind.
#[derive(Clone, Debug, Eq, Getters, PartialEq, Setters)]
pub struct Environment {
    /// The `Kind` of this environment.
    #[get = "pub"]
    current: Kind,
    /// The key-value pairs for this environment (common + kind).
    #[get = "pub"]
    props: HashMap<String, String>,
}

impl Environment {}

/// Get the default file path.
#[cfg(unix)]
pub fn get_config_path() -> Result<PathBuf> {
    let mut config_path = PathBuf::new();

    match env::var("XDG_CONFIG_HOME") {
        Ok(val) => {
            config_path.push(val);
        }
        Err(_e) => if let Some(home_dir) = env::home_dir() {
            config_path.push(home_dir);
            config_path.push(".config");
        } else {
            return Err(ErrorKind::ConfigPath.into());
        },
    }

    Ok(config_path)
}

/// Get the default file path.
#[cfg(windows)]
pub fn get_config_path() -> Result<PathBuf> {
    let mut config_path = PathBuf::new();

    match env::var("APPDATA") {
        Ok(val) => {
            config_path.push(val);
        }
        Err(_e) => if let Some(home_dir) = env::home_dir() {
            config_path.push(home_dir);
            config_path.push(".config");
        } else {
            return Err(ErrorKind::ConfigPath.into());
        },
    }

    Ok(config_path)
}

/// Read a property file into a `HashMap`.
fn read_props_file(config: &Config, props: &mut HashMap<String, String>) -> Result<()> {
    let mut file_path = get_config_path()?;
    file_path.push(config.app_name());
    let mut common_filename: String = (*config.kind()).into();
    common_filename.push_str(ENV_SUFFIX);
    file_path.push(common_filename);
    let common_file = File::open(file_path)?;
    let common_reader = BufReader::new(common_file);
    for line_res in common_reader.lines() {
        match line_res {
            Ok(line) => {
                if *config.comments() && line.starts_with(*config.comment_char()) {
                    continue;
                }
                let mut kv = Vec::new();
                for tok in line.split('=') {
                    kv.push(tok);
                }

                if kv.len() != 2 {
                    return Err(ErrorKind::InvalidProperty.into());
                }
                props.insert(kv[0].to_string(), kv[1].to_string());
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

impl TryFrom<Config> for Environment {
    type Error = Error;

    fn try_from(config: Config) -> Result<Environment> {
        let mut props: HashMap<String, String> = HashMap::new();
        if *config.os() {
            props.extend(env::vars());
        }
        if *config.common() {
            let common_config = ConfigBuilder::default()
                .app_name(config.app_name().to_string())
                .kind(Kind::Common)
                .build()?;
            read_props_file(&common_config, &mut props)?;
        }
        read_props_file(&config, &mut props)?;

        Ok(Environment {
            current: *config.kind(),
            props,
        })
    }
}
