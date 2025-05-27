use core::panic;
use std::fs;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{PathBuf, Path};
use regex::Regex;
use std::io::{BufRead, BufReader};
use std::fs::File;
use std::process::{Command, exit};
use std::collections::HashMap;

fn main() {
    //grab original command without self
    let mut args = env::args().skip(1);

    //no arguments passed?
    let Some(command) = args.next() else {
        panic!("No command to run. Was Steam launch option set to `%command%`?")
    };

    //Os Independent Vars
    let home_dir = home::home_dir().expect("Could not determine home directory");
    let mut good_config_paths = home_dir.join("Documents").join("game_configs");
    let paths_file = env::current_exe().unwrap().parent().unwrap().to_path_buf().join("paths.txt");
    let steam_app_id = env::var("SteamAppId").expect("Not running under steam");
    let log_file_path = good_config_paths.join(format!("{}_config_workaround.log", steam_app_id));
    let steam_id: String;
    let steam_id_3: String;
    const STEAM_ID_OFFSET: u64 = 76561197960265728;

    // variables needed depending on os
    //Linux only
    let win_user_path = "pfx/drive_c/users/steamuser";
    let mut proton_prefix: Option<PathBuf> = None;

    //Windows only 
    let mut custom_env: HashMap<String, String> = HashMap::new();

    //Set variables if under windows
    match std::env::consts::OS {
        "windows" => {
            let steam_id_str = env::var("STEAMID").expect("No SteamID Found");
            let steam_id_u64: u64 = steam_id_str.parse().expect("STEAMID is not a valid number");
            let steam_id_3_u64: u64 = steam_id_u64 - STEAM_ID_OFFSET;
            steam_id = steam_id_u64.to_string();
            steam_id_3 = steam_id_3_u64.to_string();
            //Get document path from registry
            let custom_documents_path = get_documents_path();
            //Override default good config path with the one from registry
            good_config_paths = custom_documents_path.join("game_configs");

            //Create custom env for shell expanding later
            custom_env.insert("STEAMID".to_string(), steam_id.to_string());
            custom_env.insert("SteamID3".to_string(), steam_id_3.to_string());
            custom_env.insert("DOCUMENTS".to_string(), custom_documents_path.to_string_lossy().to_string());
        } 
        //Set variables if under linux
        "linux" => {
            proton_prefix = Some(PathBuf::from(env::var("STEAM_COMPAT_DATA_PATH").expect("No Compat data path")));
            let steam_user = env::var("SteamUser").expect("Steam User not found!");
            //.local/share/Steam/config/config.vdf
            let config_vdf = home_dir.join(".local").join("share").join("Steam").join("config").join("config.vdf");
            //STEAMID=$(grep -Pzo '"'${SteamUser}'"\s+{\s+"SteamID"\s+"[0-9]+"' /home/${USER}/.local/share/Steam/config/config.vdf | grep --text -oP '(?<=\s")[0-9]+')
            let contents = fs::read_to_string(&config_vdf).expect("Failed to read config.vdf");
            let block_re = Regex::new(&format!(r#""{}"\s*\{{[^{{}}]*?"SteamID"\s+"([0-9]+)""#,regex::escape(&steam_user))).expect("Regex invalid");
            if let Some(caps) = block_re.captures(&contents) {
                let steam_id_str = &caps[1];
                let steam_id_u64: u64 = steam_id_str.parse().expect("STEAMID is not a valid number");
                let steam_id_3_u64: u64 = steam_id_u64 - STEAM_ID_OFFSET;  
                steam_id = steam_id_u64.to_string();
                steam_id_3 = steam_id_3_u64.to_string();
            } else {
                panic!("No steamid found. How is this possible")
            }
        }
        _ => {
            panic!("Unsupported OS at this stage???")
        }
    }

    
    
    //Create Config Dir if not exists
    fs::create_dir_all(&good_config_paths).expect("Could not create dir");

    //Find game lines in paths.txt
    let file = File::open(&paths_file).expect("No paths.txt found");
    let reader = BufReader::new(file);

    let prefix = format!("{};", steam_app_id);
    let matching_lines: Vec<String> = reader
        .lines()
        .filter_map(Result::ok) // Skip lines with errors
        .filter(|line| line.trim_start().starts_with(&prefix))
        .collect();

    //Open log file 
    let mut log_file = OpenOptions::new()
        .create(true)    // Create the file if it doesn't exist
        .append(true)    // Append to the file (don't overwrite)
        .open(log_file_path).expect("Failed to create or open logfile");


    //End of "startup" actually 


    //If no matches in path.txt execute original game without doing anything
    if matching_lines.is_empty(){
        let _ = writeln!(log_file, "No matches in path.txt, launching game without doing anything");
        Command::new(&command)
        .args(args) // Remaining launch arguments
        .status().expect("Failed to launch original game"); // Wait for game to finish
        exit(1)
    } else {
        //Copy configs to game
        process_configs(true, &matching_lines, &steam_id, &steam_id_3, &steam_app_id, proton_prefix.as_deref(), &custom_env, &win_user_path, good_config_paths.as_path(), &mut log_file);
        //Launch Game
        Command::new(&command)
        .args(args) // Remaining launch arguments
        .status().expect("Failed to launch original game"); // Wait for game to finish
        //Copy configs to good folder
        process_configs(false, &matching_lines, &steam_id, &steam_id_3, &steam_app_id, proton_prefix.as_deref(), &custom_env, &win_user_path, good_config_paths.as_path(), &mut log_file);
        exit(1)
    }

}

fn process_configs(
    to_game: bool,
    matching_lines: &[String],
    steam_id: &str,
    steam_id_3: &str,
    steam_app_id: &str,
    proton_prefix: Option<&Path>,
    custom_envs: &HashMap<String, String>,
    win_user_path: &str,
    good_config_paths: &Path,
    log_file: &mut File,
) {
    for raw_config in matching_lines {
        let mut split = raw_config.split(';');
        split.next(); // Skip app id
        let mut config_path = split.next().unwrap().to_string();
        let config = split.next().unwrap();

        let game_config_path: PathBuf;

        match std::env::consts::OS {
            "linux" => {
                config_path = config_path.replace("%APPDATA%", "Application Data")
                    .replace("%DOCUMENTS%", "Documents")
                    .replace("%USERPROFILE%", "")
                    .replace("%LOCALAPPDATA%", "AppData/Local")
                    .replace("%STEAMID%", steam_id)
                    .replace("%SteamID3%", steam_id_3);
                game_config_path = match proton_prefix {
                    Some(prefix) => prefix.join(win_user_path).join(&config_path),
                    None => {
                        panic!("Error: Proton prefix not set on Linux.");
                    }
                }
            }
            "windows" => {
                //config_path = config_path.replace("%APPDATA%", "AppData");
                //config_path = config_path.replace("%DOCUMENTS%", "Documents");
                //config_path = config_path.replace("%USERPROFILE%", "");
                //config_path = config_path.replace("%LOCALAPPDATA%", "AppData/Local");
                //config_path = config_path.replace("%STEAMID%", steam_id.as_str());
                //config_path = config_path.replace("%SteamID3%", steam_id_3.as_str());
                //Use shell expansion to replace the variables
                config_path = expand_windows_env_vars(&config_path, Some(&custom_envs));
                game_config_path = PathBuf::from(config_path);
            }
            _ => {
                panic!("OS Not Supported!")
            }
        }   
        if to_game {
            copy_configs(
            &good_config_paths.join(steam_app_id).join(config),
            &game_config_path.join(config),
            log_file,
            );
        } else {
            copy_configs(
            &game_config_path.join(config),
            &good_config_paths.join(steam_app_id).join(config),
            log_file,
            );
        } 
    }   
}

fn copy_configs(from: &Path, to: &Path, log_file: &mut std::fs::File){
    if let Err(e) = fs::copy(&from, &to){
        let _ = writeln!(log_file, "Failed to Copy {} to {}", from.to_string_lossy(), to.to_string_lossy());
        let _ = writeln!(log_file, "Error: {}", e);
    } else {
        let _ = writeln!(log_file, "Copied {} to {}", from.to_string_lossy(), to.to_string_lossy());
    }
}

#[cfg(windows)]
fn get_documents_path() -> PathBuf {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders")
        .expect("Could not open registry key for Documents folder");

    let documents: String = key.get_value("Personal")
        .expect("Could not read 'Personal' (Documents) path from registry");

    // Expand environment variables like %USERPROFILE%
    let expanded = expand_windows_env_vars(&documents, None);

    PathBuf::from(expanded.to_string())
}

#[cfg(not(windows))]
fn get_documents_path() -> PathBuf {
    panic!("get_documents_path() should only be called on Windows");
}

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


