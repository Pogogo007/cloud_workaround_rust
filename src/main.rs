use core::panic;
use std::fs;
use std::env;
use std::path::{PathBuf};
use std::io::{BufRead, BufReader};
use std::fs::File;
use std::process::{Command, exit};
use log::{debug, info, LevelFilter};
use chrono::Local;

mod platform;
mod shared;
#[cfg(windows)]
use platform::windows as platform_os;
#[cfg(unix)]
use platform::linux as platform_os;

fn main() {
    //grab original command without self
    let mut args = env::args().skip(1);

    //no arguments passed?
    let Some(command) = args.next() else {
        panic!("No command to run. Was Steam launch option set to `%command%`?")
    };

    //setup before so the logging starts as soon as possible
    let good_config_paths: PathBuf = platform_os::get_good_config_paths();
    
    //Os Independent Vars
    let paths_file = env::current_exe().unwrap().parent().unwrap().to_path_buf().join("paths.txt");
    let steam_app_id = env::var("SteamAppId").expect("Not running under steam");
    let steam_user = env::var("SteamUser").expect("Steam User not found!");
    
    
    let log_file_path = good_config_paths.join(format!("{}_config_workaround.log", steam_app_id));
    setup_logging(log_file_path);

    //INIT Main OS Specific
    let (steam_id, steam_id_3) = platform_os::init(&steam_user);
    

    //Startup finished. Log debug info
    debug!("Startup finished. Logging debug info");
    debug!("Operating System:  {}", std::env::consts::OS );
    debug!("Steam App ID:  {}", steam_app_id );
    debug!("Home Directory: {}", home::home_dir().unwrap().to_string_lossy());
    debug!("Good Config Directory: {}", good_config_paths.to_string_lossy());
    debug!("Paths file: {}", paths_file.to_string_lossy());
    debug!("SteamID64: {}", steam_id);
    debug!("SteamID3: {}", steam_id_3);
    platform_os::print_debug();
    //Finish debug logging

    //Create Config Dir if not exists
    fs::create_dir_all(&good_config_paths).expect("Could not create config dir");

    //Find game lines in paths.txt
    let file = File::open(&paths_file).expect("No paths.txt found");
    let reader = BufReader::new(file);

    let prefix = format!("{};", steam_app_id);
    let matching_lines: Vec<String> = reader
        .lines()
        .filter_map(Result::ok) // Skip lines with errors
        .filter(|line| line.trim_start().starts_with(&prefix))
        .collect();

    //If no matches in path.txt execute original game without doing anything
    if matching_lines.is_empty(){
        info!("No matches in path.txt found for game id {}, launching game without copying", steam_app_id);
        Command::new(&command)
        .args(args) // Remaining launch arguments
        //Launch as child since we dont need to keep running
        .spawn().expect("Failed to launch");
        exit(1)
    } else {
        info!("Restoring configs to game: {}", steam_app_id);
        //Copy configs to game
        platform_os::process_configs(true, &matching_lines, &steam_id, &steam_id_3, &steam_app_id, good_config_paths.as_path());
        info!("Finished restoring to game: {}, launching...", steam_app_id);
        //Launch Game
        Command::new(&command)
        .args(args) // Remaining launch arguments
        .status().expect("Failed to launch original game"); // Wait for game to finish
        //Copy configs to good folder
        info!("Game: {} exited. Backing up config files.", steam_app_id);
        platform_os::process_configs(false, &matching_lines, &steam_id, &steam_id_3, &steam_app_id, good_config_paths.as_path());
        info!("Finished backing up config files. for game: {}. Exiting.", steam_app_id);
        exit(1)
    }

}

fn setup_logging(log_file_path: PathBuf){
    //Hook panic and expect
    //Panic and expect are not the best ideas but for a program that needs everything else before to continue running its fine
    std::panic::set_hook(Box::new(|info| {
        log::error!("Panic: {}", info);
    }));
    //Loglevel depending on build
    let log_level = if cfg!(debug_assertions) {
        LevelFilter::Debug // Debug build
    } else {
        LevelFilter::Info // Release build
    };
    //Setup fern
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}]",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log_level)
        .chain(std::io::stdout())
        .chain(fern::log_file(log_file_path).expect("Unable to access log file"))
        .apply().expect("Unable to create logger");
}