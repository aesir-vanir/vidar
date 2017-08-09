//! `vidar` test setup
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use vidar::Kind;
use vidar::error::Result;

pub const COMMON: &'static str = r"key1=val1
key2=val2
key3=val3";
pub const DEV: &'static str = r"url=https://localhost";
pub const INT: &'static str = r"";
pub const TEST: &'static str = r"# This is a comment
url=https://testurl.vidar.com";
pub const STAGE: &'static str = r"this is a bad property";
pub const PROD: &'static str = r"url=https://produrl.vidar.com
creds=secret";

fn create_file(kind: Kind, contents: &str, path: &mut PathBuf) -> Result<()> {
    let file_name = match kind {
        Kind::Common => "common.env",
        Kind::Development => "dev.env",
        Kind::Integration => "int.env",
        Kind::Test => "test.env",
        Kind::Staging => "stage.env",
        Kind::Production => "prod.env",
    };

    path.push(file_name);
    let common = File::create(&path)?;
    let mut writer = BufWriter::new(common);
    writer.write_all(contents.as_bytes())?;
    path.pop();

    Ok(())
}

pub fn setup(subfolder: &str, content_map: Option<HashMap<Kind, &str>>) -> Result<()> {
    let mut path = env::temp_dir();
    path.push(subfolder);
    fs::create_dir_all(&path)?;

    if let Some(content_map) = content_map {
        for (kind, content) in content_map {
            create_file(kind, content, &mut path)?;
        }
    } else {
        create_file(Kind::Common, COMMON, &mut path)?;
        create_file(Kind::Development, DEV, &mut path)?;
        create_file(Kind::Integration, INT, &mut path)?;
        create_file(Kind::Test, TEST, &mut path)?;
        create_file(Kind::Staging, STAGE, &mut path)?;
        create_file(Kind::Production, PROD, &mut path)?;
    }

    env::set_current_dir(&path)?;
    Ok(())
}

pub fn teardown(subfolder: &str) -> Result<()> {
    let mut path = env::temp_dir();
    path.push(subfolder);
    fs::remove_dir_all(path)?;
    Ok(())
}

macro_rules! wrap {
    ($sub:expr, $map:expr, $test:block) => {
        lifecycle::setup($sub, $map).expect("Setup failed!");
        $test
        lifecycle::teardown($sub).expect("Teardown failed!");
    };
}
