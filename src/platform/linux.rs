use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::env;
use log::{debug};
use crate::shared;

//Linux only
const WINE_USER_PATH: &str = "pfx/drive_c/users/steamuser";
static PROTON_PREFIX: OnceLock<PathBuf> = OnceLock::new();

pub fn get_good_config_paths() -> PathBuf{
    return home::home_dir().unwrap().join("Documents").join("game_configs");
}

pub fn init(steam_user: &str) -> (String, String) {
    PROTON_PREFIX.set(PathBuf::from(env::var("STEAM_COMPAT_DATA_PATH").expect("No proton compat data folder found. This tool does not support linux native games!")));
    //.local/share/Steam/config/config.vdf
    let config_vdf_linux = home::home_dir().unwrap().join(".local").join("share").join("Steam").join("config").join("config.vdf");
    let steam_id = shared::get_steamid_from_config(config_vdf_linux, &steam_user);
    let steam_id_3_u64: u64 = steam_id.parse::<u64>().expect("SteamID is not a number!") - shared::STEAM_ID_OFFSET;  
    let steam_id_3 = steam_id_3_u64.to_string();
    return (steam_id, steam_id_3);   
}

pub fn print_debug(){
    debug!("Proton Prefix: {}", PROTON_PREFIX.get().unwrap().to_string_lossy().to_string());
}

pub fn process_configs(
    to_game: bool,
    matching_lines: &Vec<String>,
    _steam_id: &str,
    _steam_id_3: &str,
    steam_app_id: &str,
    good_config_paths: &Path
) {
    for raw_config in matching_lines {
        
        let (config_path, config) = shared::process_raw_config_line(raw_config);

        let game_config_path: PathBuf;

        let config_path = config_path.replace("%APPDATA%", "Application Data")
            .replace("%DOCUMENTS%", "Documents")
            .replace("%USERPROFILE%", "")
            .replace("%LOCALAPPDATA%", "AppData/Local")
            .replace("%STEAMID%", _steam_id)
            .replace("%SteamID3%", _steam_id_3);
        
        game_config_path = PROTON_PREFIX.get().unwrap().join(WINE_USER_PATH).join(config_path);

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