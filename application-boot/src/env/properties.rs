use config::Case::Snake;
use config::{Config, ConfigError, Environment};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BootstrapProperties {
    pub application: ApplicationProperties,
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
pub struct DiscoveryProperties {
    pub address: String,
    pub token: Option<String>,
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
                    locations: None,
                    file_names: None,
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
