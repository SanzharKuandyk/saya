use std::fs::{self, File};
use std::io::BufReader;
use std::path::PathBuf;

use saya_config::Config;
use serde::{Deserialize, Serialize};
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
