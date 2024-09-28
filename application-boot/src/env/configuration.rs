use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use crate::env::properties::{CloudConfigProperties, CloudProperties};
use application_core::env::environment::{ApplicationEnvironment, Environment};
use application_core::env::property_resolver::PropertyResolver;
use async_std::task::block_on;
use async_trait::async_trait;
use config::Case::Snake;
use config::Config;
use consulrs::client::{ConsulClient, ConsulClientSettingsBuilder};
use consulrs::kv;
use tempfile::{tempdir, TempDir};
use tracing::info;

pub struct Configuration {}

#[async_trait]
pub trait ConfigurationResolver {
    fn read_native_config_from_environment(
        env: &ApplicationEnvironment,
    ) -> Result<Config, Box<dyn Error>> {
        let activate_profiles = &env.get_active_profiles();
        let config_locations = &env.get_config_locations();
        let config_file_names = &env.get_file_names();

        let mut builder = Config::builder();
        let config_files =
            Self::get_native_config_files(activate_profiles, config_locations, config_file_names);
        for config_file in config_files {
            builder = builder.add_source(config::File::with_name(&config_file));
        }
        builder = builder.add_source(
            config::Environment::default()
                .separator("_")
                .convert_case(Snake)
                .try_parsing(true),
        );

        let config = builder.build().unwrap();
        Ok(config)
    }

    fn read_remote_config_from_environment(
        env: &ApplicationEnvironment,
    ) -> Result<Config, Box<dyn Error>> {
        let activate_profiles = &env.get_active_profiles();
        let mut builder = Config::builder();
        let cloud_properties = env.get_property::<CloudProperties>("application.cloud");

        let dir = tempdir()?;
        if let Some(cloud) = &cloud_properties {
            if let Some(cloud_config) = &cloud.config {
                if cloud_config.enabled {
                    let application_name = env.get_property::<String>("application.name").unwrap();
                    let config_files = Self::get_remote_config_files(
                        cloud_config,
                        activate_profiles,
                        &application_name,
                        &dir,
                    );
                    let config_files = block_on(config_files);
                    if let Ok(config_files) = config_files {
                        for config_file in config_files {
                            builder = builder.add_source(config::File::from(config_file));
                        }
                    }
                }
            }
        }
        let config = builder.build().unwrap();
        Ok(config)
    }

    fn get_native_config_files(
        activate_profiles: &Vec<String>,
        config_locations: &Option<Vec<String>>,
        config_file_names: &Option<Vec<String>>,
    ) -> Vec<String> {
        let mut config_files = Vec::new();
        if let Some(locations) = config_locations {
            for location in locations {
                if let Some(file_names) = config_file_names {
                    for file_name in file_names {
                        let dot_index = file_name.find('.').unwrap();
                        for profile in activate_profiles {
                            let mut full_name = location.to_string() + "/" + file_name;
                            if profile != "default" {
                                full_name = location.to_string()
                                    + "/"
                                    + &full_name[0..dot_index]
                                    + "-"
                                    + profile
                                    + &full_name[dot_index..];
                            }
                            let path = Path::new(&full_name);
                            if path.exists() {
                                config_files.push(full_name.clone());
                            }
                        }
                    }
                }
            }
        }
        if config_files.is_empty() {
            let path = Path::new("./config.toml");
            if path.exists() {
                config_files.push("./config.toml".to_string());
            }
        }
        config_files
    }

    async fn get_remote_config_files(
        cloud_config: &CloudConfigProperties,
        activate_profiles: &Vec<String>,
        application_name: &str,
        dir: &TempDir,
    ) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let client = ConsulClient::new(
            ConsulClientSettingsBuilder::default()
                .address(&cloud_config.address)
                .token(cloud_config.token.clone().unwrap_or_default())
                .build()
                .unwrap(),
        )
        .unwrap();
        let mut config_files = Vec::new();
        for profile in activate_profiles {
            let mut key: String = profile.to_string() + "/" + application_name;
            if profile == "default" {
                key = application_name.to_string();
            }
            let result = kv::raw(&client, &key, None).await;
            match result {
                Ok(result) => {
                    // write to temp file
                    let vec = result.response;
                    let file_path = dir.path().join(application_name.to_string() + ".toml");
                    fs::write(&file_path, vec)?;
                    config_files.push(file_path);
                }
                Err(e) => {
                    info!(
                        "application {} config not found on cloud, {:?}",
                        application_name, e
                    );
                }
            }
        }

        Ok(config_files)
    }
}

impl ConfigurationResolver for Configuration {}
