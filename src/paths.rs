use std::{env, path::PathBuf};

pub fn data_dir() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("CWCTL_HOME") {
        return Ok(PathBuf::from(path));
    }
    if let Some(path) = env::var_os("XDG_DATA_HOME") {
        return Ok(PathBuf::from(path).join("cwctl"));
    }

    let home = if cfg!(windows) {
        env::var_os("APPDATA")
            .or_else(|| env::var_os("LOCALAPPDATA"))
            .or_else(|| env::var_os("USERPROFILE"))
            .or_else(
                || match (env::var_os("HOMEDRIVE"), env::var_os("HOMEPATH")) {
                    (Some(drive), Some(path)) => {
                        let mut home = PathBuf::from(drive);
                        home.push(path);
                        Some(home.into_os_string())
                    }
                    _ => None,
                },
            )
            .or_else(|| env::var_os("HOME"))
    } else {
        env::var_os("HOME")
            .or_else(|| env::var_os("USERPROFILE"))
            .or_else(
                || match (env::var_os("HOMEDRIVE"), env::var_os("HOMEPATH")) {
                    (Some(drive), Some(path)) => {
                        let mut home = PathBuf::from(drive);
                        home.push(path);
                        Some(home.into_os_string())
                    }
                    _ => None,
                },
            )
    };

    home.map(|path| {
        let path = PathBuf::from(path);
        if cfg!(windows)
            && (env::var_os("APPDATA").is_some() || env::var_os("LOCALAPPDATA").is_some())
        {
            path.join("cwctl")
        } else {
            path.join(".local/share/cwctl")
        }
    })
    .ok_or_else(|| "error: unable to determine the user data directory".to_owned())
}
