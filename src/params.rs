//! AppParams are used inside `main()` to initialize values to user-defined
//! specifications.
//! Typically loaded from the dotfile associated with the project. On Linux
//! systems supporting XDG specifications, this defaults to
//! `$XDG_DATA_HOME/lady/params.json`. On Windows, this defaults to
//! `%APPDATA%\lady\params.json`.

use crate::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::{Error, Read};
use std::path::{Path, PathBuf};

const DEFAULT_PORT: i16 = 3306;

/// Retrieves the base directory where Lady's config will reside. Depends on the
/// target operating system of the build.
fn base_dir() -> Option<PathBuf> {
    // check for override directory
    // if LADY_CONFIG_DIR is not defined, move on to the default directory
    debug!("looking for $LADY_CONFIG_DIR");
    if let Some(dir) = env::var_os("LADY_CONFIG_DIR") {
        // repackage the directory
        return Some(PathBuf::from(dir));
    }

    #[cfg(target_os = "windows")]
    {
        // %APPDATA% (e.g. C:\Users\Name\AppData\Roaming)
        debug!("finding $APPDATA definition");
        return env::var_os("APPDATA").map(PathBuf::from);
    }

    #[cfg(target_os = "macos")]
    {
        // ~/Library/Application Support
        debug!("finding ~/Library/Application Support");
        return env::var_os("HOME").map(|home| {
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
        });
    }

    // defaults to Linux convention. Hopefully this includes other OSes as well.
    // $XDG_DATA_HOME, falling back to ~/.local/share.
    // The XDG spec says empty or relative values must be ignored.
    debug!("checking for $XDG_DATA_HOME");
    return env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| {
            debug!("$XDG_DATA_HOME failed, checking ~/.local/share");
            env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("share"))
        });
}

fn get_file_path() -> PathBuf {
    let base_dir = base_dir().expect("could not determine base directory");
    debug!("determined base directory: {base_dir:?}");

    base_dir.join("lady").join("params.json")
}

fn read_file(path: &Path) -> Result<String, Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}

/// A struct of all paramaters for the app. Typically loaded from the dotfile
/// associated with the project. On Linux supporting XDG specifications, this
/// defaults to `$XDG_DATA_HOME/lady/params.json`. On Windows, this defaults to
/// `%APPDATA%\lady\params.json`.
#[derive(Serialize, Deserialize)]
pub struct AppParams {
    pub port: i16,
}
impl AppParams {
    /// Attempts to load the `AppParam` from a JSON file, but an IO error can
    /// occur. It is recommended to use `default()` in that case.
    pub fn from_json() -> Result<Self, Error> {
        let param_path = get_file_path();
        info!("Found AppParam path at {param_path:?}");

        let contents = read_file(&param_path)?;

        // type annotations
        let params: Self = serde_json::from_str(&contents)?;
        Ok(params)
    }
    /// Creates a new AppParams file with default values.
    pub fn default() -> Self {
        Self { port: DEFAULT_PORT }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_ne;

    use super::*;

    #[test]
    fn base_dir_specified() {
        unsafe {
            env::set_var("LADY_CONFIG_DIR", "/etc/lady");
        }
        let dir = base_dir().unwrap();
        assert_eq!(dir, PathBuf::from("/etc/lady"));
    }

    #[test]
    fn base_dir_with_os() {
        // Try with --all-targets to test properly
        assert_ne!(base_dir(), None);
    }
}
