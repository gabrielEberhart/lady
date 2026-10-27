//! AppParams are used inside `main()` to initialize values to user-defined
//! specifications.
//! Typically loaded from the dotfile associated with the project. On Linux
//! systems supporting XDG specifications, this defaults to
//! `$XDG_DATA_HOME/lady/params.json`. On Windows, this defaults to
//! `%APPDATA%\lady\params.json`.

use std::env;
use std::path::PathBuf;

/// Retrieves the base directory where Lady's config will reside. Depends on the
/// target operating system of the build.
fn base_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        // %APPDATA% (e.g. C:\Users\Name\AppData\Roaming)
        env::var_os("APPDATA").map(PathBuf::from)
    }

    #[cfg(target_os = "macos")]
    {
        // ~/Library/Application Support
        env::var_os("HOME").map(|home| {
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
        })
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        // $XDG_DATA_HOME, falling back to ~/.local/share.
        // The XDG spec says empty or relative values must be ignored.
        env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| {
                env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("share"))
            })
    }
}

fn get_file_path() -> PathBuf {
    let base_dir = base_dir().expect("could not determine base directory");
    base_dir.join("lady").join("params.json")
}

/// A struct of all paramaters for the app. Typically loaded from the dotfile
/// associated with the project. On Linux supporting XDG specifications, this
/// defaults to `$XDG_DATA_HOME/lady/params.json`. On Windows, this defaults to
/// `%APPDATA%\lady\params.json`.
pub struct AppParams {
    pub port: i16,
}
impl AppParams {
    pub fn load_from_file() -> Self {
        let param_path = get_file_path();
        let mut params: Self;
        todo!("deserialize params.json");
        params
    }
}
