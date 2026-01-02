use std::{
    collections::HashMap,
    net::IpAddr,
    path::{Path, PathBuf},
    str::FromStr,
};

use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use getset::CloneGetters;
use rocket::data::{ByteUnit, Limits};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Clone, Debug, CloneGetters)]
#[serde(rename_all = "snake_case")]
#[getset(get_clone = "pub")]
pub struct TlsConfig {
    key: PathBuf,
    certs: PathBuf,
}

impl From<TlsConfig> for rocket::config::TlsConfig {
    fn from(value: TlsConfig) -> Self {
        rocket::config::TlsConfig::from_paths(value.key.clone(), value.certs.clone())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, CloneGetters)]
#[serde(rename_all = "snake_case")]
#[getset(get_clone = "pub")]
pub struct LimitsConfig {
    #[serde(default = "LimitsConfig::_d_files")]
    files: ByteUnit,

    #[serde(default)]
    file_types: HashMap<String, ByteUnit>,
}

impl LimitsConfig {
    fn _d_files() -> ByteUnit {
        10 * ByteUnit::GiB
    }

    pub fn extension_limit(&self, extension: impl AsRef<str>) -> ByteUnit {
        let extension = extension.as_ref().to_string();
        let ext = extension.trim_start_matches(".").to_string();
        if let Some(specific) = self.file_types.get(&ext) {
            *specific
        } else {
            self.files()
        }
    }
}

impl From<LimitsConfig> for Limits {
    fn from(value: LimitsConfig) -> Self {
        let limits = Limits::default().limit("file", value.files());
        value
            .file_types()
            .into_iter()
            .fold(limits, |target, (ext, amt)| {
                target.limit(format!("file/${ext}"), amt)
            })
    }
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            files: Self::_d_files(),
            file_types: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, CloneGetters)]
#[serde(rename_all = "snake_case")]
#[getset(get_clone = "pub")]
pub struct ServerConfig {
    #[serde(default = "ServerConfig::_d_addr")]
    address: IpAddr,

    #[serde(default = "ServerConfig::_d_port")]
    port: u16,

    #[serde(default)]
    secret_key: Option<String>,

    #[serde(default)]
    tls: Option<TlsConfig>,

    #[serde(default)]
    limits: LimitsConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: Self::_d_addr(),
            port: Self::_d_port(),
            secret_key: None,
            tls: None,
            limits: Default::default(),
        }
    }
}

impl ServerConfig {
    fn _d_addr() -> IpAddr {
        IpAddr::from_str("0.0.0.0").unwrap()
    }

    fn _d_port() -> u16 {
        8080
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, CloneGetters)]
#[serde(rename_all = "snake_case")]
#[getset(get_clone = "pub")]
pub struct AuthConfig {
    #[serde(default = "AuthConfig::_d_admin_user")]
    admin_user: String,

    #[serde(default = "AuthConfig::_d_admin_password")]
    admin_password: String,
}

impl AuthConfig {
    fn _d_admin_user() -> String {
        String::from("admin")
    }

    fn _d_admin_password() -> String {
        String::from("admin")
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            admin_user: Self::_d_admin_user(),
            admin_password: Self::_d_admin_password(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, CloneGetters)]
#[serde(rename_all = "snake_case")]
#[getset(get_clone = "pub")]
pub struct DatabaseConfig {
    url: String,

    #[serde(default = "DatabaseConfig::_d_database", alias = "db")]
    database: String,
}

impl DatabaseConfig {
    fn _d_database() -> String {
        String::from("abyssal")
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: String::from("mongodb://localhost:27017"),
            database: Self::_d_database(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, CloneGetters)]
#[serde(rename_all = "snake_case")]
#[getset(get_clone = "pub")]
pub struct FilesystemRootConfig {
    #[serde(default)]
    display_name: Option<String>,
    path: PathBuf,
}

impl Default for FilesystemRootConfig {
    fn default() -> Self {
        Self {
            display_name: Some("Root".to_string()),
            path: "/".into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, CloneGetters)]
#[serde(rename_all = "snake_case")]
#[getset(get_clone = "pub")]
pub struct FilesystemConfig {
    /// Root directory of the entire file server
    #[serde(default = "FilesystemConfig::_d_filesystem")]
    filesystem: PathBuf,

    /// Allow modification of roots from the web UI
    #[serde(default = "FilesystemConfig::_d_allow_root_modification")]
    allow_root_modification: bool,

    /// Filesystem roots to automatically create/configure
    #[serde(default = "FilesystemConfig::_d_directories")]
    directories: HashMap<String, FilesystemRootConfig>,
}

impl FilesystemConfig {
    fn _d_filesystem() -> PathBuf {
        PathBuf::from("/")
    }

    fn _d_allow_root_modification() -> bool {
        true
    }

    fn _d_directories() -> HashMap<String, FilesystemRootConfig> {
        HashMap::from_iter(vec![("root".to_string(), FilesystemRootConfig::default())])
    }
}

impl Default for FilesystemConfig {
    fn default() -> Self {
        Self {
            filesystem: Self::_d_filesystem(),
            allow_root_modification: Self::_d_allow_root_modification(),
            directories: Self::_d_directories()
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, CloneGetters, Default)]
#[serde(rename_all = "snake_case")]
#[getset(get_clone = "pub")]
pub struct Config {
    #[serde(default)]
    server: ServerConfig,

    #[serde(default, alias = "auth")]
    authentication: AuthConfig,

    #[serde(default, alias = "db")]
    database: DatabaseConfig,

    #[serde(default, alias = "fs")]
    filesystem: FilesystemConfig,
}

impl Config {
    pub fn load(files: impl IntoIterator<Item = impl AsRef<Path>>) -> crate::Result<Self> {
        let fig = files
            .into_iter()
            .fold(Figment::new(), |target, source| {
                target.merge(Toml::file(source))
            })
            .merge(Env::prefixed("ABYSSAL_").split("__"));
        Ok(fig.extract::<Self>()?)
    }

    pub fn rocket_config(&self) -> Figment {
        rocket::Config::figment()
            .join(("address", self.server().address()))
            .join(("port", self.server().port()))
            .join(("ident", "abyssal::rocket"))
            .join(("secret_key", self.server().secret_key()))
            .join(("tls", self.server().tls()))
            .join(("limits", Limits::from(self.server().limits())))
            .join(("full_config", self.clone()))
            .join((
                "databases",
                json!({
                    "mongodb": {
                        "url": self.database().url(),
                        "database": self.database().database()
                    }
                }),
            ))
    }
}
