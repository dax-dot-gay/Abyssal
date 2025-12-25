use std::{
    net::IpAddr,
    path::{Path, PathBuf},
    str::FromStr,
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TlsConfig {
    pub key: PathBuf,
    pub certs: PathBuf,
}

impl From<TlsConfig> for rocket::config::TlsConfig {
    fn from(value: TlsConfig) -> Self {
        rocket::config::TlsConfig::from_paths(value.key.clone(), value.certs.clone())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ServerConfig {
    #[serde(default = "ServerConfig::_d_addr")]
    pub address: IpAddr,

    #[serde(default = "ServerConfig::_d_port")]
    pub port: u16,

    #[serde(default)]
    pub secret_key: Option<String>,

    #[serde(default)]
    pub tls: Option<TlsConfig>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: Self::_d_addr(),
            port: Self::_d_port(),
            secret_key: None,
            tls: None,
        }
    }
}

impl ServerConfig {
    fn _d_addr() -> IpAddr {
        IpAddr::from_str("0.0.0.0").unwrap()
    }

    fn _d_port() -> u16 {
        5174
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AuthConfig {
    #[serde(default = "AuthConfig::_d_admin_user")]
    pub admin_user: String,

    #[serde(default = "AuthConfig::_d_admin_password")]
    pub admin_password: String,
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

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,

    #[serde(default)]
    pub authentication: AuthConfig,
}
