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
    pub password_change_expiration_ns: u128,
    pub email_change_expiration_ns: u128,
    pub database_path: String,
    pub storage_path: String,
    pub assets_path: String,
    pub tmp_path: String,
}

pub const FIELD_ADDRESS: &'static str = "address";
pub const FIELD_BIND: &'static str = "bind";
pub const FIELD_SECRET: &'static str = "secret";
pub const FIELD_INVITE_EXPIRATION: &'static str = "invite_expiration_ns";
pub const FIELD_PASSWORD_CHANGE_EXPIRATION: &'static str = "password_change_expiration_ns";
pub const FIELD_EMAIL_CHANGE_EXPIRATION: &'static str = "email_change_expiration_ns";
pub const FIELD_DATABASE_PATH: &'static str = "database_path";
pub const FIELD_STORAGE_PATH: &'static str = "storage_path";
pub const FIELD_ASSETS_PATH: &'static str = "assets_path";
pub const FIELD_TMP_PATH: &'static str = "tmp_path";

impl ApiConfig {
    pub async fn new(conf_path: impl AsRef<Path>) -> Self {
        read_or_create(conf_path, || ApiConfig::default()).await
    }

    pub async fn new_with_path_override(
        conf_path: impl AsRef<Path>,
        database_path: impl Into<String>,
        storage_path: impl Into<String>,
        assets_path: impl Into<String>,
        tmp_path: impl Into<String>,
    ) -> Self {
        read_or_create(conf_path, || {
            let mut conf = ApiConfig::default();
            let database_path = database_path.into();
            let storage_path = storage_path.into();
            let assets_path = assets_path.into();
            let tmp_path = tmp_path.into();

            if !database_path.is_empty() {
                conf.database_path = database_path;
            }

            if !storage_path.is_empty() {
                conf.storage_path = storage_path;
            }

            if !assets_path.is_empty() {
                conf.assets_path = assets_path;
            }

            if !tmp_path.is_empty() {
                conf.tmp_path = tmp_path;
            }

            conf
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

    let tmp_path = Path::new(&config_file.tmp_path);
    if !tmp_path.exists() {
        fs::create_dir_all(tmp_path).await.unwrap()
    }

    let assets_path = Path::new(&config_file.assets_path);
    if !assets_path.exists() {
        panic!("assets path not found: {:?}", assets_path);
    }

    config_file
}

#[tokio::test]
async fn test_api_config_new() {
    init_log();

    let test_config = async |conf_path: &str,
                             db_path: &str,
                             storage_path: &str,
                             assets_path: &str,
                             tmp_path: &str| {
        let confa = read_or_create(conf_path, || {
            let mut conf = ApiConfig::default();
            conf.database_path = db_path.to_string();
            conf.storage_path = storage_path.to_string();
            conf.assets_path = assets_path.to_string();
            conf.tmp_path = tmp_path.to_string();

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
        "/tmp",
        "/tmp/test_catconf/tmp",
    )
    .await;
    test_config(
        "../target/tmp/catconf/catsquad.conf",
        "../target/tmp/catconf/db",
        "../target/tmp/catconf/storage",
        "/tmp",
        "../target/tmp/catconf/tmp",
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
            password_change_expiration_ns: 1800000000000,
            email_change_expiration_ns: 1800000000000,
            database_path: "target/db".to_string(),
            storage_path: "target/storage".to_string(),
            assets_path: "target/dist".to_string(),
            tmp_path: "target/tmp".to_string(),
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
                FIELD_PASSWORD_CHANGE_EXPIRATION => {
                    conf.password_change_expiration_ns =
                        if let Ok(v) = u128::from_str_radix(value, 10) {
                            v
                        } else {
                            continue;
                        }
                }
                FIELD_EMAIL_CHANGE_EXPIRATION => {
                    conf.password_change_expiration_ns =
                        if let Ok(v) = u128::from_str_radix(value, 10) {
                            v
                        } else {
                            continue;
                        }
                }
                FIELD_DATABASE_PATH => conf.database_path = value.to_string(),
                FIELD_STORAGE_PATH => conf.storage_path = value.to_string(),
                FIELD_ASSETS_PATH => conf.assets_path = value.to_string(),
                FIELD_TMP_PATH => conf.tmp_path = value.to_string(),
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
        push(
            &mut output,
            FIELD_PASSWORD_CHANGE_EXPIRATION,
            &value.password_change_expiration_ns.to_string(),
        );
        output.push('\n');
        push(
            &mut output,
            FIELD_EMAIL_CHANGE_EXPIRATION,
            &value.password_change_expiration_ns.to_string(),
        );
        output.push('\n');
        push(&mut output, FIELD_DATABASE_PATH, &value.database_path);
        output.push('\n');
        push(&mut output, FIELD_STORAGE_PATH, &value.storage_path);
        output.push('\n');
        push(&mut output, FIELD_ASSETS_PATH, &value.assets_path);
        output.push('\n');
        push(&mut output, FIELD_TMP_PATH, &value.tmp_path);

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
        "{}=hello\n{}=111\n{}=hello2\n{}=123\n{}=124\n{}=124\n{}=hello3\n{}=tmp\n{}=tmp2\n{}=tmp3",
        FIELD_ADDRESS,
        FIELD_BIND,
        FIELD_SECRET,
        FIELD_INVITE_EXPIRATION,
        FIELD_PASSWORD_CHANGE_EXPIRATION,
        FIELD_EMAIL_CHANGE_EXPIRATION,
        FIELD_DATABASE_PATH,
        FIELD_STORAGE_PATH,
        FIELD_ASSETS_PATH,
        FIELD_TMP_PATH,
    );
    let settings = ApiConfig::from(&input);
    let settings_str = settings.to_string();
    assert_eq!(input, settings_str);
}
