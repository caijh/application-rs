use config::Case::Snake;
use config::{Config, ConfigError, Environment};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug)]
/// 启动配置属性结构体
///
/// 该结构体包含了应用配置和日志配置两个重要的属性，
/// 是应用启动时加载的配置信息的载体。
pub struct BootstrapProperties {
    /// 应用配置属性
    /// 包含了应用运行所需的各种基础配置
    pub application: ApplicationProperties,
    /// 日志配置属性
    /// 包含了日志记录的相关配置，如日志级别、日志文件路径等
    pub logger: LoggerProperties,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoggerProperties {
    pub enabled: bool,
    pub level: String,
    pub file: String,
    pub log_dir: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigProperties {
    pub activate: ConfigActivateProperties,
    pub locations: Option<Vec<String>>,
    pub file_names: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigActivateProperties {
    pub profiles: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CloudProperties {
    pub discovery: Option<DiscoveryProperties>,
    pub config: Option<CloudConfigProperties>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerProperties {
    pub address: String,
    pub token: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HostProperties {
    pub ip: String,
    pub port: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DiscoveryProperties {
    pub server: ServerProperties,
    pub host: Option<HostProperties>,
    pub service: Option<ServiceProperties>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServiceProperties {
    pub check: ServiceCheckProperties,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServiceCheckProperties {
    pub address: String,
    pub interval: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CloudConfigProperties {
    pub enabled: bool,
    pub address: String,
    pub token: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApplicationProperties {
    pub name: String,
    pub port: Option<u16>,
    pub config: ConfigProperties,
    pub cloud: Option<CloudProperties>,
}

impl Default for BootstrapProperties {
    fn default() -> Self {
        BootstrapProperties {
            application: ApplicationProperties {
                name: "".to_string(),
                port: None,
                config: ConfigProperties {
                    activate: ConfigActivateProperties {
                        profiles: vec!["default".to_string()],
                    },
                    locations: Some(vec![".".to_string()]),
                    file_names: Some(vec!["config.toml".to_string()]),
                },
                cloud: None,
            },
            logger: LoggerProperties {
                enabled: false,
                level: "info".to_string(),
                file: "info".to_string(),
                log_dir: "./logs".to_string(),
            },
        }
    }
}

impl BootstrapProperties {
    pub fn read_from_path(path: &str) -> Result<BootstrapProperties, ConfigError> {
        if Path::new(path).exists() {
            let builder = Config::builder()
                .add_source(config::File::with_name(path))
                .add_source(
                    Environment::default()
                        .separator("_")
                        .convert_case(Snake)
                        .try_parsing(true),
                );
            let config = builder.build()?;
            let properties = config.try_deserialize::<BootstrapProperties>()?;
            Ok(properties)
        } else {
            Ok(BootstrapProperties::default())
        }
    }

    pub fn get_application_name(&self) -> String {
        self.application.name.clone()
    }

    pub fn get_application_port(&self) -> u16 {
        self.application.port.unwrap_or(0)
    }
}
