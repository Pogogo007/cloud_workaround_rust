use core::panic;
use std::fs;
use std::path::{Path, PathBuf};
use regex::Regex;
use log::{error, info};


// Contains code used by both platform modules

pub const STEAM_ID_OFFSET: u64 = 76561197960265728;

pub fn get_steamid_from_config(config_path: PathBuf, steam_user: &str) -> String{
    //STEAMID=$(grep -Pzo '"'${SteamUser}'"\s+{\s+"SteamID"\s+"[0-9]+"' /home/${USER}/.local/share/Steam/config/config.vdf | grep --text -oP '(?<=\s")[0-9]+')
    let contents = fs::read_to_string(&config_path).expect("Failed to read config.vdf");
    let block_re = Regex::new(&format!(r#""{}"\s*\{{[^{{}}]*?"SteamID"\s+"([0-9]+)""#,regex::escape(&steam_user))).expect("Regex invalid");
    if let Some(caps) = block_re.captures(&contents) { 
        let steam_id: String = caps[1].to_string();
        return steam_id;
    } else {
        panic!("No steamid found in config.vdf. How is this possible?")
    }
}

pub fn copy_configs(from: &Path, to: &Path){
    if let Err(e) = fs::copy(&from, &to){
        error!("Failed to Copy {} to {}", from.to_string_lossy(), to.to_string_lossy());
        error!("Error: {}", e);
    } else {
        info!("Copied {} to {}", from.to_string_lossy(), to.to_string_lossy());
    }
}

pub fn process_raw_config_line(raw_config: &String) -> (&str, &str) {
    let mut split = raw_config.split(';');
    split.next(); // Skip app id
    let config_path: &str = split.next().unwrap();
    let config: &str = split.next().unwrap();
    return (config_path, config)
}