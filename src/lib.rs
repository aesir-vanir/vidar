//! Environment Mapping
#![feature(try_from)]
#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate getset;

use error::{Error, ErrorKind, Result};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub mod error;

/// Suffix for environment variables file name.
const ENV_SUFFIX: &'static str = ".env";

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
#[builder(default, setter(into))]
pub struct Config {
    /// The environment `Kind` we are loading.
    #[get = "pub"]
    #[set = "pub"]
    kind: Kind,
    /// Should we read from a `common.env` file?
    #[get = "pub"]
    #[set = "pub"]
    common: Option<bool>,
    /// Should we recursively search up directories for the files?
    #[get = "pub"]
    #[set = "pub"]
    recursive: Option<bool>,
    /// The base directory to look for files.
    #[get = "pub"]
    #[set = "pub"]
    base_dir: Option<PathBuf>,
    /// Does the property file have comments?
    #[get = "pub"]
    #[set = "pub"]
    comments: Option<bool>,
    /// The comment character.
    #[get = "pub"]
    #[set = "pub"]
    comment_char: Option<char>,
}

impl Config {
    /// Create a new `Config` for the given `Kind`.
    pub fn new(kind: Kind) -> Config {
        Config {
            kind: kind,
            common: Some(true),
            recursive: None,
            base_dir: None,
            comments: None,
            comment_char: None,
        }
    }
}

impl Default for Config {
    fn default() -> Config {
        Config::new(Kind::Common)
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

/// Read a property file into a `HashMap`.
fn read_props_file(config: &Config, props: &mut HashMap<String, String>) -> Result<()> {
    let mut file_path = env::current_dir()?;
    let mut common_filename: String = (*config.kind()).into();
    common_filename.push_str(ENV_SUFFIX);
    file_path.push(common_filename);
    let common_file = File::open(file_path)?;
    let common_reader = BufReader::new(common_file);
    for line_res in common_reader.lines() {
        match line_res {
            Ok(line) => {
                if let Some(true) = *config.comments() {
                    if let Some(comment_char) = *config.comment_char() {
                        if line.starts_with(comment_char) {
                            continue;
                        }
                    }
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

impl<'a> TryFrom<&'a str> for Environment {
    type Error = Error;

    fn try_from(name: &'a str) -> Result<Environment> {
        let mut props: HashMap<String, String> = HashMap::new();
        let current: Kind = TryFrom::try_from(name)?;
        let mut config: Config = Default::default();
        read_props_file(&config, &mut props)?;
        config.set_kind(current);
        read_props_file(&config, &mut props)?;

        Ok(Environment {
            current: current,
            props: props,
        })
    }
}

impl TryFrom<Config> for Environment {
    type Error = Error;

    fn try_from(config: Config) -> Result<Environment> {
        let mut props: HashMap<String, String> = HashMap::new();
        if let Some(true) = *config.common() {
            let common_config: Config = Default::default();
            read_props_file(&common_config, &mut props)?;
        }
        read_props_file(&config, &mut props)?;

        Ok(Environment {
            current: *config.kind(),
            props: props,
        })
    }
}

#[cfg(test)]
mod tests {
    use error::{Error, ErrorKind};
    use std::collections::HashMap;
    use std::convert::TryFrom;
    use super::{Config, ConfigBuilder, Environment, Kind};

    #[test]
    fn no_file() {
        match Environment::try_from("int") {
            Ok(_) => assert!(false),
            Err(e) => match e {
                Error(ErrorKind::Io(_), _) => assert!(true),
                _ => assert!(false),
            },
        }
    }

    #[test]
    fn invalid_property() {
        match Environment::try_from("stage") {
            Ok(_) => assert!(false),
            Err(e) => match e {
                Error(ErrorKind::InvalidProperty, _) => assert!(true),
                _ => assert!(false),
            },
        }
    }

    fn check_test_props(props: &HashMap<String, String>) {
        assert!(props.contains_key("key1"));
        assert_eq!(props.get(&"key1".to_string()), Some(&"val1".to_string()));
        assert!(props.contains_key("key2"));
        assert_eq!(props.get(&"key2".to_string()), Some(&"val2".to_string()));
        assert!(props.contains_key("key3"));
        assert_eq!(props.get(&"key3".to_string()), Some(&"val3".to_string()));
        assert!(props.contains_key("url"));
    }

    fn check_env_str(name: &str, url_value: &str) {
        match Environment::try_from(name) {
            Ok(env) => {
                let props = env.props();
                check_test_props(&props);
                assert_eq!(props.get(&"url".to_string()), Some(&url_value.to_string()));
            }
            Err(_) => assert!(false),
        }
    }

    fn check_env_config(config: Config, url_value: &str) {
        match Environment::try_from(config) {
            Ok(env) => {
                let props = env.props();
                check_test_props(&props);
                assert_eq!(props.get(&"url".to_string()), Some(&url_value.to_string()));
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn dev_env() {
        check_env_str("dev", "https://localhost");
    }

    #[test]
    fn dev_config_env() {
        let mut config: Config = Default::default();
        config.set_kind(Kind::Development);
        check_env_config(config, "https://localhost");
    }

    #[test]
    fn test_with_comments_env() {
        let config = ConfigBuilder::default()
            .kind(Kind::Test)
            .comments(true)
            .comment_char('#')
            .build()
            .expect("Unable to build Config");
        check_env_config(config, "https://testurl.vidar.com");
    }

    #[test]
    fn prod_env() {
        check_env_str("prod", "https://produrl.vidar.com");
    }

    #[test]
    fn prod_config_env() {
        let mut config: Config = Default::default();
        config.set_kind(Kind::Production);
        check_env_config(config, "https://produrl.vidar.com");
    }
}
