use std::{
    io,
    path::{Path, PathBuf},
    str::FromStr,
};

use catsquad_log::prelude::*;
use tokio::fs;

#[derive(Clone, PartialEq, Debug)]
pub struct ApiConfig {
    pub address: String,
    pub bind: String,
    pub secret: String,
    pub invite_expiration_ns: u128,
    pub database_path: String,
    pub storage_path: String,
}

pub const FIELD_ADDRESS: &'static str = "address";
pub const FIELD_BIND: &'static str = "bind";
pub const FIELD_SECRET: &'static str = "secret";
pub const FIELD_INVITE_EXPIRATION: &'static str = "invite_expiration_ns";
pub const FIELD_DATABASE_PATH: &'static str = "database_path";
pub const FIELD_STORAGE_PATH: &'static str = "storage_path";

impl ApiConfig {
    pub async fn new(
        conf_path: impl AsRef<Path>,
        database_path: impl Into<String>,
        storage_path: impl Into<String>,
    ) -> Self {
        read_or_create(conf_path, || ApiConfig {
            database_path: database_path.into(),
            storage_path: storage_path.into(),
            ..Default::default()
        })
        .await
    }
}

async fn read_or_create(path: impl AsRef<Path>, or_else: impl FnOnce() -> ApiConfig) -> ApiConfig {
    let file_path = path.as_ref();

    let config_file = fs::read_to_string(file_path).await;
    let config_file = match config_file {
        Ok(v) => ApiConfig::from(&v),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            let path_parent = file_path.parent().and_then(|parent| {
                let v = parent.to_str();
                match v {
                    Some(v) if !v.is_empty() => Some(parent),
                    _ => None,
                }
            });

            if let Some(parent) = path_parent
                && !parent.exists()
            {
                fs::create_dir_all(parent).await.unwrap();
            }

            let value = or_else();
            fs::write(file_path, value.to_string()).await.unwrap();
            value
        }
        Err(err) => panic!("{err}"),
    };

    let database_path = Path::new(&config_file.database_path);
    if !database_path.exists() {
        fs::create_dir_all(database_path).await.unwrap()
    }

    let storage_path = Path::new(&config_file.storage_path);
    if !storage_path.exists() {
        fs::create_dir_all(storage_path).await.unwrap()
    }

    config_file
}

#[tokio::test]
async fn test_api_config_new() {
    init_log();

    let test_config = async |conf_path: &str, db_path: &str, storage_path: &str| {
        let confa = read_or_create(conf_path, || {
            let mut conf = ApiConfig::default();
            conf.database_path = db_path.to_string();
            conf.storage_path = storage_path.to_string();

            conf
        })
        .await;
        let confb = fs::read_to_string(conf_path).await.unwrap();
        let confb = ApiConfig::from(&confb);
        assert_eq!(confa, confb);
        fs::remove_file(conf_path).await.unwrap();
        fs::remove_dir(db_path).await.unwrap();
        fs::remove_dir(storage_path).await.unwrap();
    };

    test_config(
        "/tmp/test_catconf/catsquad.conf",
        "/tmp/test_catconf/db",
        "/tmp/test_catconf/storage",
    )
    .await;
    test_config(
        "../target/tmp/catconf/catsquad.conf",
        "../target/tmp/catconf/db",
        "../target/tmp/catconf/storage",
    )
    .await;
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            address: "http://localhost:3000".to_string(),
            bind: "localhost:3000".to_string(),
            secret: "test".to_string(),
            invite_expiration_ns: 1800000000000,
            database_path: "db".to_string(),
            storage_path: "storage".to_string(),
        }
    }
}

impl<T: AsRef<str>> From<T> for ApiConfig {
    fn from(value: T) -> Self {
        let mut conf = ApiConfig::default();

        for line in value.as_ref().split('\n') {
            let line = line.trim();
            let (name, value) = match line.split_once('=') {
                Some((name, value)) if !name.is_empty() && !value.is_empty() => (name, value),
                _ => continue,
            };

            match name {
                FIELD_ADDRESS => conf.address = value.to_string(),
                FIELD_BIND => conf.bind = value.to_string(),
                FIELD_SECRET => conf.secret = value.to_string(),
                FIELD_INVITE_EXPIRATION => {
                    conf.invite_expiration_ns = if let Ok(v) = u128::from_str_radix(value, 10) {
                        v
                    } else {
                        continue;
                    }
                }
                FIELD_DATABASE_PATH => conf.database_path = value.to_string(),
                FIELD_STORAGE_PATH => conf.storage_path = value.to_string(),
                _ => (),
            }
        }

        conf
    }
}

impl From<&ApiConfig> for String {
    fn from(value: &ApiConfig) -> Self {
        let mut output = String::new();

        let push = |output: &mut String, field: &str, value: &str| {
            output.push_str(field);
            output.push('=');
            output.push_str(value);
        };

        push(&mut output, FIELD_ADDRESS, &value.address);
        output.push('\n');
        push(&mut output, FIELD_BIND, &value.bind);
        output.push('\n');
        push(&mut output, FIELD_SECRET, &value.secret);
        output.push('\n');
        push(
            &mut output,
            FIELD_INVITE_EXPIRATION,
            &value.invite_expiration_ns.to_string(),
        );
        output.push('\n');
        push(&mut output, FIELD_DATABASE_PATH, &value.database_path);
        output.push('\n');
        push(&mut output, FIELD_STORAGE_PATH, &value.storage_path);

        output
    }
}

impl From<ApiConfig> for String {
    fn from(value: ApiConfig) -> Self {
        String::from(&value)
    }
}

impl ToString for ApiConfig {
    fn to_string(&self) -> String {
        String::from(self)
    }
}

#[test]
fn test_api_config_from_str() {
    let input = format!(
        "{}=hello\n{}=111\n{}=hello2\n{}=123\n{}=hello3\n{}=tmp",
        FIELD_ADDRESS,
        FIELD_BIND,
        FIELD_SECRET,
        FIELD_INVITE_EXPIRATION,
        FIELD_DATABASE_PATH,
        FIELD_STORAGE_PATH
    );
    let settings = ApiConfig::from(&input);
    let settings_str = settings.to_string();
    assert_eq!(input, settings_str);
}
