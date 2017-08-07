//! Environment Mapping
#![feature(try_from)]
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

mod error;

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
fn read_props_file(prefix: &str, kvs: &mut HashMap<String, String>) -> Result<()> {
    let mut file_path = env::current_dir()?;
    let mut common_filename = String::from(prefix);
    common_filename.push_str(ENV_SUFFIX);
    file_path.push(common_filename);
    let common_file = File::open(file_path)?;
    let common_reader = BufReader::new(common_file);
    for line_res in common_reader.lines() {
        if let Ok(line) = line_res {
            let mut kv = Vec::new();
            for tok in line.split('=') {
                kv.push(tok);
            }

            if kv.len() != 2 {
                return Err(ErrorKind::InvalidProperty.into());
            }
            kvs.insert(kv[0].to_string(), kv[1].to_string());
        }
    }
    Ok(())
}

impl<'a> TryFrom<&'a str> for Environment {
    type Error = Error;

    fn try_from(name: &'a str) -> Result<Environment> {
        let mut props: HashMap<String, String> = HashMap::new();
        let current: Kind = TryFrom::try_from(name)?;
        read_props_file(Kind::Common.into(), &mut props)?;
        read_props_file(current.into(), &mut props)?;

        Ok(Environment {
               current: current,
               props: props,
           })
    }
}

#[cfg(test)]
mod tests {
    use error::{Error, ErrorKind};
    use std::convert::TryFrom;
    use std::io::{self, Write};
    use super::Environment;

    #[test]
    fn no_file_io_error() {
        match Environment::try_from("int") {
            Ok(_) => assert!(false),
            Err(e) => {
                match e {
                    Error(ErrorKind::Io(_), _) => assert!(true),
                    _ => assert!(false),
                }
            }
        }
    }

    #[test]
    fn bad_props_invalid_property() {
        match Environment::try_from("stage") {
            Ok(_) => assert!(false),
            Err(e) => {
                match e {
                    Error(ErrorKind::InvalidProperty, _) => assert!(true),
                    _ => assert!(false),
                }
            }
        }
    }

    #[test]
    fn invalid_kind() {
        match Environment::try_from("blah") {
            Ok(_) => assert!(false),
            Err(e) => {
                match e {
                    Error(ErrorKind::InvalidKind(_), _) => assert!(true),
                    _ => assert!(false),
                }
            }
        }
    }

    #[test]
    fn dev_env() {
        match Environment::try_from("dev") {
            Ok(env) => {
                let kvs = env.props();
                assert!(kvs.contains_key("key1"));
                assert_eq!(kvs.get(&"key1".to_string()), Some(&"val1".to_string()));
                assert!(kvs.contains_key("key2"));
                assert_eq!(kvs.get(&"key2".to_string()), Some(&"val2".to_string()));
                assert!(kvs.contains_key("key3"));
                assert_eq!(kvs.get(&"key3".to_string()), Some(&"val3".to_string()));
                assert!(kvs.contains_key("url"));
                assert_eq!(kvs.get(&"url".to_string()),
                           Some(&"https://localhost".to_string()));
            }
            Err(e) => {
                writeln!(io::stderr(), "{}", e).expect("");
                assert!(false)
            }
        }
    }

    #[test]
    fn test_env() {
        match Environment::try_from("test") {
            Ok(env) => {
                let kvs = env.props();
                assert!(kvs.contains_key("key1"));
                assert_eq!(kvs.get(&"key1".to_string()), Some(&"val1".to_string()));
                assert!(kvs.contains_key("key2"));
                assert_eq!(kvs.get(&"key2".to_string()), Some(&"val2".to_string()));
                assert!(kvs.contains_key("key3"));
                assert_eq!(kvs.get(&"key3".to_string()), Some(&"val3".to_string()));
                assert!(kvs.contains_key("url"));
                assert_eq!(kvs.get(&"url".to_string()),
                           Some(&"https://testurl.vidar.com".to_string()));
            }
            Err(e) => {
                writeln!(io::stderr(), "{}", e).expect("");
                assert!(false)
            }
        }
    }

    #[test]
    fn prod_env() {
        match Environment::try_from("prod") {
            Ok(env) => {
                let kvs = env.props();
                assert!(kvs.contains_key("key1"));
                assert_eq!(kvs.get(&"key1".to_string()), Some(&"val1".to_string()));
                assert!(kvs.contains_key("key2"));
                assert_eq!(kvs.get(&"key2".to_string()), Some(&"val2".to_string()));
                assert!(kvs.contains_key("key3"));
                assert_eq!(kvs.get(&"key3".to_string()), Some(&"val3".to_string()));
                assert!(kvs.contains_key("creds"));
                assert_eq!(kvs.get(&"creds".to_string()), Some(&"secret".to_string()));
                assert!(kvs.contains_key("url"));
                assert_eq!(kvs.get(&"url".to_string()),
                           Some(&"https://produrl.vidar.com".to_string()));
            }
            Err(e) => {
                writeln!(io::stderr(), "{}", e).expect("");
                assert!(false)
            }
        }
    }
}
