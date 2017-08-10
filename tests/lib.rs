#![feature(try_from)]
extern crate vidar;

#[macro_use]
mod lifecycle;

use std::collections::HashMap;
use std::convert::TryFrom;
use vidar::{Config, ConfigBuilder, Environment, Error, ErrorKind, Kind};

#[test]
fn no_file() {
    let mut most = HashMap::new();
    most.insert(Kind::Common, lifecycle::COMMON);
    most.insert(Kind::Development, lifecycle::DEV);
    most.insert(Kind::Test, lifecycle::TEST);
    most.insert(Kind::Staging, lifecycle::STAGE);
    most.insert(Kind::Production, lifecycle::PROD);

    let config = ConfigBuilder::default()
        .app_name("no_file")
        .kind(Kind::Integration)
        .build()
        .expect("Unable to build `Config`");

    wrap!("no_file", Some(most), {
        match Environment::try_from(config) {
            Ok(_) => assert!(false),
            Err(e) => match e {
                Error(ErrorKind::Io(_), _) => assert!(true),
                _ => assert!(false),
            },
        }
    });
}

#[test]
fn invalid_property() {
    let config = ConfigBuilder::default()
        .app_name("invalid_property")
        .kind(Kind::Staging)
        .build()
        .expect("Unable to build `Config`");
    wrap!("invalid_property", None, {
        match Environment::try_from(config) {
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

fn check_env_config(folder_name: &str, config: Config, url_value: &str) {
    wrap!(folder_name, None, {
        match Environment::try_from(config) {
            Ok(env) => {
                let props = env.props();
                check_test_props(props);
                assert_eq!(props.get(&"url".to_string()), Some(&url_value.to_string()));
            }
            Err(_e) => assert!(false),
        }
    });
}

#[test]
fn dev_config_env() {
    let config = ConfigBuilder::default()
        .app_name("dev_config")
        .common(true)
        .build()
        .expect("Unable to build Config");
    check_env_config("dev_config", config, "https://localhost");
}

#[test]
fn test_with_comments_env() {
    let config = ConfigBuilder::default()
        .app_name("test_with_comments")
        .kind(Kind::Test)
        .common(true)
        .comments(true)
        .comment_char('#')
        .build()
        .expect("Unable to build Config");
    check_env_config("test_with_comments", config, "https://testurl.vidar.com");
}

#[test]
fn prod_config_env() {
    let config = ConfigBuilder::default()
        .app_name("prod_config")
        .common(true)
        .kind(Kind::Production)
        .build()
        .expect("Unable to build Config");
    check_env_config("prod_config", config, "https://produrl.vidar.com");
}
