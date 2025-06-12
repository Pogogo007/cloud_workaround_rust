use winreg::enums::*;
use winreg::{RegKey, types::FromRegValue, HKEY};
use std::path::{PathBuf};
use regex::Regex;
use std::collections::HashMap;


/*fn get_documents_path() -> PathBuf {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders")
        .expect("Could not open registry key for Documents folder");

    let documents: String = key.get_value("Personal")
        .expect("Could not read 'Personal' (Documents) path from registry");

    // Expand environment variables like %USERPROFILE%
    let expanded = expand_windows_env_vars(&documents, None);

    PathBuf::from(expanded.to_string())
}
*/

pub fn get_regkey_value<T>(key_path: &str, key_name: &str) -> Result<T, std::io::Error>
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
pub fn expand_windows_env_vars(input: &str, overrides: Option<&HashMap<String, String>>) -> String {
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

