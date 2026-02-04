use std::fs::{self, File};
use std::io::BufReader;
use std::path::PathBuf;

use saya_config::Config;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use windows::Win32::UI::Shell::{FOLDERID_RoamingAppData, KF_FLAG_DEFAULT, SHGetKnownFolderPath};

/// Load the default config shipped in the repo
fn load_repo_default_config() -> anyhow::Result<Config> {
    tracing::info!("Loading repo default config...");
    let file = File::open("config.json")?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}

/// Windows Roaming folder
fn roaming_dir() -> PathBuf {
    unsafe {
        let path = SHGetKnownFolderPath(&FOLDERID_RoamingAppData, KF_FLAG_DEFAULT, None)
            .expect("Failed to get RoamingAppData");
        PathBuf::from(path.to_string().unwrap())
    }
}

fn saya_root() -> PathBuf {
    roaming_dir().join("Saya")
}

fn profiles_dir() -> PathBuf {
    saya_root().join("profiles")
}

/// Represents a user profile
#[derive(Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub value: Config,
}

/// Initialize user config folders and main profile if missing
pub fn init_user_config() -> anyhow::Result<()> {
    fs::create_dir_all(profiles_dir())?;

    let main_profile = profiles_dir().join("main.json");

    if !main_profile.exists() {
        // Use repo default as the initial main profile
        let default_config = load_repo_default_config()?;
        let profile = Profile {
            name: "main".into(),
            value: default_config,
        };
        fs::write(&main_profile, serde_json::to_string_pretty(&profile)?)?;
        tracing::info!("Created main profile in Roaming");
    }

    Ok(())
}

/// Load a user profile by name, defaulting to main if name not found
pub fn load_user_profile(name: &str) -> anyhow::Result<Config> {
    let profile_file = profiles_dir().join(format!("{name}.json"));

    if profile_file.exists() {
        let data = fs::read_to_string(profile_file)?;
        let profile: Profile = serde_json::from_str(&data)?;
        Ok(profile.value)
    } else {
        tracing::warn!("Profile {name} not found, falling back to main profile or repo default");
        let main_file = profiles_dir().join("main.json");
        if main_file.exists() {
            let data = fs::read_to_string(main_file)?;
            let profile: Profile = serde_json::from_str(&data)?;
            Ok(profile.value)
        } else {
            // First-run fallback to repo default
            load_repo_default_config()
        }
    }
}

/// Add a new profile cloned from main (or repo default if main missing)
pub fn add_profile_from_default(new_name: &str) -> anyhow::Result<PathBuf> {
    let default_config = load_user_profile("main")?;
    let profile = Profile {
        name: new_name.into(),
        value: default_config,
    };
    let file = profiles_dir().join(format!("{new_name}.json"));
    fs::write(&file, serde_json::to_string_pretty(&profile)?)?;
    tracing::info!("Created new profile: {new_name}");
    Ok(file)
}

pub fn save_config(config: Config, profile_name: &str) -> anyhow::Result<()> {
    let profile_file = profiles_dir().join(format!("{profile_name}.json"));

    let profile = Profile {
        name: profile_name.to_string(),
        value: config,
    };

    fs::write(&profile_file, serde_json::to_string_pretty(&profile)?)?;
    Ok(())
}

pub fn update_config_field(config: &mut Config, key: &str, value: &str) -> anyhow::Result<()> {
    // Convert config to serde_json::Value
    let mut json_val = serde_json::to_value(&mut *config)?;

    // Split dotted keys like "ocr.capture_region.x"
    let parts: Vec<&str> = key.split('.').collect();

    // Recursive helper to set value
    fn set_value(obj: &mut Map<String, Value>, parts: &[&str], value: &str) {
        if parts.is_empty() {
            return;
        }

        let k = parts[0];

        if parts.len() == 1 {
            // Last part: update the value based on existing type
            if let Some(existing) = obj.get_mut(k) {
                match existing {
                    Value::Bool(_) => *existing = Value::Bool(value == "true"),
                    Value::Number(_) => {
                        if let Ok(v) = value.parse::<i64>() {
                            *existing = Value::Number(v.into());
                        }
                    }
                    Value::String(_) => *existing = Value::String(value.to_string()),
                    _ => *existing = Value::String(value.to_string()), // fallback
                }
            } else {
                // Key does not exist, create as string
                obj.insert(k.to_string(), Value::String(value.to_string()));
            }
        } else {
            // Traverse nested object
            if let Some(Value::Object(map)) = obj.get_mut(k) {
                set_value(map, &parts[1..], value);
            } else {
                // Create nested object if missing
                let mut new_map = Map::new();
                set_value(&mut new_map, &parts[1..], value);
                obj.insert(k.to_string(), Value::Object(new_map));
            }
        }
    }

    if let Value::Object(ref mut map) = json_val {
        set_value(map, &parts, value);
    }

    // Convert back to Config
    *config = serde_json::from_value(json_val)?;
    Ok(())
}
