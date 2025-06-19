use winreg::enums::*;
use winreg::{RegKey, types::FromRegValue, HKEY};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use log::{warn, debug};
use crate::shared;

const DOCUMENTS_REGKEY_PATH: &str = "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders";
const DOCUMENTS_REGKEY: &str = "Personal";
const STEAM_REGKEY_PATH: &str = "HKEY_LOCAL_MACHINE\\SOFTWARE\\WOW6432Node\\Valve\\Steam";
const STEAM_REGKEY: &str = "InstallPath";

static CUSTOM_ENV: OnceLock<HashMap<String, String>> = OnceLock::new();
//const default_config_vdf_windows: PathBuf = PathBuf::from("C:/Program Files (x86)/Steam/config/config.vdf");
static DOCUMENTS_PATH: OnceLock<PathBuf> = OnceLock::new();

//static STEAM_PATH: OnceLock<PathBuf> = OnceLock::new();
const DEFAULT_STEAM_PATH: &str = "C:\\Program Files (x86)\\Steam\\";

pub fn get_good_config_paths() -> PathBuf{
    DOCUMENTS_PATH.get_or_init(|| {
        get_regkey_value::<String>(DOCUMENTS_REGKEY_PATH, DOCUMENTS_REGKEY)
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                warn!("No Documents Path set in registry, falling back to default path");
                home::home_dir().expect("No Home Directory Found!").join("Documents")
            })
    }).join("game_configs")
}

//Main entrypoint MUST BE CALLED!
pub fn init(steam_user: &str) -> (String, String) {
    let steam_path: PathBuf = get_regkey_value::<String>(STEAM_REGKEY_PATH, STEAM_REGKEY)
    .map(PathBuf::from)
    .unwrap_or_else(|_| {
        warn!("No steam path found in registry, falling back to default path");
        return PathBuf::from(DEFAULT_STEAM_PATH);
    });

    let steam_id = env::var("STEAMID").unwrap_or_else(|_| {
                warn!("SteamID not set via env vars on windows. Falling back to config parsing");
                return shared::get_steamid_from_config(steam_path.join("config").join("config.vdf"), &steam_user);
    });
    let steam_id_3_u64: u64 = steam_id.parse::<u64>().expect("msg") - shared::STEAM_ID_OFFSET;
    let steam_id_3 = steam_id_3_u64.to_string();

    let mut custom_env: HashMap<String, String> = HashMap::new();
    //Create custom env for shell expanding later
    custom_env.insert("STEAMID".to_string(), steam_id.to_string());
    custom_env.insert("SteamID3".to_string(), steam_id_3.to_string());
    custom_env.insert("DOCUMENTS".to_string(), DOCUMENTS_PATH.get().unwrap().to_string_lossy().to_string());
    CUSTOM_ENV.set(custom_env).expect("Custom env already set? How is this possible!");
    return (steam_id, steam_id_3)
}

pub fn print_debug() {
    for (k, v) in CUSTOM_ENV.get().unwrap() {
        debug!("{} = {}", k, v);
    }
}

fn get_regkey_value<T>(key_path: &str, key_name: &str) -> Result<T, std::io::Error>
where
    T: FromRegValue,
{
    let predef_index = key_path.find('\\').expect("Wrong Path given for registry");
    let (predef_path, subkey_path) = key_path.split_at(predef_index);
    let subkey_path = &subkey_path[1..];
    let hkcu = RegKey::predef(predef_from_str(predef_path)?);
    let key = hkcu.open_subkey(subkey_path)?;
    return key.get_value::<T, _>(key_name)
}

// Expand %% windows vars with vars from the hashmap is exists otherwise from environment
fn expand_windows_env_vars(input: &str, overrides: Option<&HashMap<String, String>>) -> String {
    let re = Regex::new(r"%([^%]+)%").unwrap();
    re.replace_all(input, |caps: &regex::Captures| {
        let key = &caps[1];
        overrides
            .and_then(|map| map.get(key).cloned())
            .or_else(|| std::env::var(key).ok())
            .unwrap_or_default()
    }).to_string()
}


fn predef_from_str(s: &str) -> Result<HKEY, std::io::Error> {
    match s.to_uppercase().as_str() {
        "HKEY_CLASSES_ROOT" => Ok(HKEY_CLASSES_ROOT),
        "HKEY_CURRENT_USER" => Ok(HKEY_CURRENT_USER),
        "HKEY_LOCAL_MACHINE" => Ok(HKEY_LOCAL_MACHINE),
        "HKEY_USERS" => Ok(HKEY_USERS),
        "HKEY_CURRENT_CONFIG" => Ok(HKEY_CURRENT_CONFIG),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Unknown registry root",
        )),
    }
}

pub fn process_configs(
    to_game: bool,
    matching_lines: &Vec<String>,
    _steam_id: &str,
    _steam_id_3: &str,
    steam_app_id: &str,
    good_config_paths: &Path
    //proton_prefix: Option<&Path>,
    //custom_envs: &HashMap<String, String>,
    //win_user_path: &str,
) {
    for raw_config in matching_lines {
        
        let (config_path, config) = shared::process_raw_config_line(raw_config);

        let game_config_path: PathBuf;

        let config_path = expand_windows_env_vars(config_path, Some(CUSTOM_ENV.get().unwrap()));
        game_config_path = PathBuf::from(config_path);

        if to_game {
            shared::copy_configs(
            &good_config_paths.join(steam_app_id).join(config),
            &game_config_path.join(config)
            );
        } else {
            shared::copy_configs(
            &game_config_path.join(config),
            &good_config_paths.join(steam_app_id).join(config)
            );
        } 
    }   
}
