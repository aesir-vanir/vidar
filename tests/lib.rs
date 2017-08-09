#![feature(try_from)]
extern crate vidar;

#[macro_use]
mod lifecycle;

use std::convert::TryFrom;
use vidar::Environment;
use vidar::error::{Error, ErrorKind};

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
