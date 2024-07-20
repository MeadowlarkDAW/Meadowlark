use std::{fs::File, io::Write};

use anyhow::Context;

const CONFIG_FILE_NAME: &str = "config.toml";

/// The configuration of the main application.
///
/// This config is saved to a config file in the TOML format.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    /// Whether or not the pointer should be locked in
    /// place when dragging a knob/slider.
    pub pointer_locking_enabled: bool,

    /// The scale factor to use for the main window.
    ///
    /// If this is `None`, then the system's scale factor
    /// will be used.
    pub main_window_scale_factor: Option<f32>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            pointer_locking_enabled: true,
            main_window_scale_factor: None,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        match Self::try_load() {
            Ok(config) => {
                log::info!("Successfully loaded config file");
                config
            }
            Err(e) => {
                let error_is_file_not_found = if let Some(e) = e.downcast_ref::<std::io::Error>() {
                    e.kind() == std::io::ErrorKind::NotFound
                } else {
                    false
                };

                if error_is_file_not_found {
                    log::warn!(
                        "Could not find config file (this is normal for first-time launches)"
                    );
                } else {
                    log::error!("Failed to load config file: {}", e);
                }

                let default_config = Self::default();

                default_config.save();

                default_config
            }
        }
    }

    fn try_load() -> anyhow::Result<Self> {
        let proj_dirs = project_directories()?;

        let contents = std::fs::read_to_string(proj_dirs.config_dir().join(CONFIG_FILE_NAME))?;

        toml::from_str::<Self>(&contents).context("Failed to parse config file")
    }

    pub fn save(&self) {
        if let Err(e) = self.try_save() {
            log::error!("Failed to save config file: {}", e);
        } else {
            log::info!("Successfully saved config file");
        }
    }

    fn try_save(&self) -> anyhow::Result<()> {
        let contents = toml::to_string_pretty(self).context("Failed to serialize config struct")?;

        let proj_dirs = project_directories()?;

        std::fs::create_dir_all(proj_dirs.config_dir())
            .context("Could not create project config directory")?;

        let mut file = File::options()
            .write(true)
            .append(false)
            .truncate(true)
            .create(true)
            .open(proj_dirs.config_dir().join(CONFIG_FILE_NAME))
            .context("Could not open file for writing")?;

        file.write_all(contents.as_bytes())
            .context("Error while writing to config file")?;

        Ok(())
    }
}

pub fn project_directories() -> anyhow::Result<directories::ProjectDirs> {
    let app_name = if crate::IS_NIGHTLY {
        "Meadowlark-Nightly"
    } else {
        "Meadowlark"
    };

    directories::ProjectDirs::from("app", "Meadowlark", app_name).ok_or_else(|| {
        anyhow::anyhow!(
            "The directories crate did not return project directories for app.Meadowlark.{}",
            app_name
        )
    })
}
