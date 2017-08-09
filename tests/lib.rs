#![feature(try_from)]
extern crate vidar;

#[macro_use]
mod lifecycle;

use std::collections::HashMap;
use std::convert::TryFrom;
use vidar::{Config, ConfigBuilder, Environment, Kind};
use vidar::error::{Error, ErrorKind};

#[test]
fn no_file() {
    let mut most = HashMap::new();
    most.insert(Kind::Common, lifecycle::COMMON);
    most.insert(Kind::Development, lifecycle::DEV);
    most.insert(Kind::Test, lifecycle::TEST);
    most.insert(Kind::Staging, lifecycle::STAGE);
    most.insert(Kind::Production, lifecycle::PROD);

    wrap!("no_file", Some(most), {
        match Environment::try_from("int") {
            Ok(_) => assert!(false),
            Err(e) => match e {
                Error(ErrorKind::Io(_), _) => assert!(true),
                _ => assert!(false),
            },
        }
    });
}

#[test]
fn invalid_kind() {
    wrap!("invalid_kind", None, {
        match Environment::try_from("blah") {
            Ok(_) => assert!(false),
            Err(e) => match e {
                Error(ErrorKind::InvalidKind(_), _) => assert!(true),
                _ => assert!(false),
            },
        }
    });
}

#[test]
fn invalid_property() {
    wrap!("invalid_property", None, {
        match Environment::try_from("stage") {
            Ok(_) => assert!(false),
            Err(e) => match e {
                Error(ErrorKind::InvalidProperty, _) => assert!(true),
                _ => assert!(false),
            },
        }
    });
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

fn check_env_str(folder_name: &str, name: &str, url_value: &str) {
    wrap!(folder_name, None, {
        match Environment::try_from(name) {
            Ok(env) => {
                let props = env.props();
                check_test_props(props);
                assert_eq!(props.get(&"url".to_string()), Some(&url_value.to_string()));
            }
            Err(_) => assert!(false),
        }
    });
}

fn check_env_config(folder_name: &str, config: Config, url_value: &str) {
    wrap!(folder_name, None, {
        match Environment::try_from(config) {
            Ok(env) => {
                let props = env.props();
                check_test_props(props);
                assert_eq!(props.get(&"url".to_string()), Some(&url_value.to_string()));
            }
            Err(_) => assert!(false),
        }
    });
}

#[test]
fn dev_env() {
    check_env_str("dev_env", "dev", "https://localhost");
}

#[test]
fn prod_env() {
    check_env_str("prod_env", "prod", "https://produrl.vidar.com");
}

#[test]
fn dev_config_env() {
    let mut config: Config = Default::default();
    config.set_kind(Kind::Development);
    check_env_config("dev_config_env", config, "https://localhost");
}

#[test]
fn test_with_comments_env() {
    let config = ConfigBuilder::default()
        .kind(Kind::Test)
        .comments(true)
        .comment_char('#')
        .build()
        .expect("Unable to build Config");
    check_env_config(
        "test_with_comments_env",
        config,
        "https://testurl.vidar.com",
    );
}

#[test]
fn prod_config_env() {
    let mut config: Config = Default::default();
    config.set_kind(Kind::Production);
    check_env_config("prod_config_env", config, "https://produrl.vidar.com");
}
