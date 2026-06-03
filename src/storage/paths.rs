use std::path::{Path, PathBuf};

const APP_DIR: &str = "tsuji";

/// Resolves the storage root directory using the precedence:
/// `cli_root` > `TSUJI_ROOT` (`tsuji_root`) > `$XDG_DATA_HOME/tsuji` > `$HOME/.local/share/tsuji`.
///
/// Empty strings in env values are treated as unset. This function is pure for
/// testability; see [`resolve_root_from_env`] for the version that reads the
/// real process environment.
pub fn resolve_root(
    cli_root: Option<&Path>,
    tsuji_root: Option<&str>,
    xdg_data_home: Option<&str>,
    home: Option<&str>,
) -> PathBuf {
    if let Some(p) = cli_root {
        return p.to_path_buf();
    }
    if let Some(v) = tsuji_root.filter(|s| !s.is_empty()) {
        return PathBuf::from(v);
    }
    if let Some(v) = xdg_data_home.filter(|s| !s.is_empty()) {
        return PathBuf::from(v).join(APP_DIR);
    }
    let home_dir = home.unwrap_or(".");
    PathBuf::from(home_dir)
        .join(".local")
        .join("share")
        .join(APP_DIR)
}

/// Like [`resolve_root`] but pulls `TSUJI_ROOT`, `XDG_DATA_HOME`, and `HOME`
/// from the real process environment.
pub fn resolve_root_from_env(cli_root: Option<&Path>) -> PathBuf {
    let tsuji_root = std::env::var("TSUJI_ROOT").ok();
    let xdg = std::env::var("XDG_DATA_HOME").ok();
    let home = std::env::var("HOME").ok();
    resolve_root(
        cli_root,
        tsuji_root.as_deref(),
        xdg.as_deref(),
        home.as_deref(),
    )
}

/// Builds the path of a channel's JSON Lines file under `root`.
pub fn channel_path(root: &Path, channel: &str) -> PathBuf {
    root.join(format!("{channel}.jsonl"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_root_has_highest_priority() {
        let p = resolve_root(
            Some(Path::new("/from/cli")),
            Some("/env"),
            Some("/xdg"),
            Some("/home"),
        );
        assert_eq!(p, PathBuf::from("/from/cli"));
    }

    #[test]
    fn tsuji_root_wins_over_xdg_and_home() {
        let p = resolve_root(None, Some("/env/tsuji"), Some("/xdg"), Some("/home"));
        assert_eq!(p, PathBuf::from("/env/tsuji"));
    }

    #[test]
    fn xdg_appends_tsuji_subdir() {
        let p = resolve_root(None, None, Some("/xdg/data"), Some("/home"));
        assert_eq!(p, PathBuf::from("/xdg/data/tsuji"));
    }

    #[test]
    fn falls_back_to_home_local_share_when_nothing_set() {
        let p = resolve_root(None, None, None, Some("/home/me"));
        assert_eq!(p, PathBuf::from("/home/me/.local/share/tsuji"));
    }

    #[test]
    fn empty_strings_in_env_are_treated_as_unset() {
        let p = resolve_root(None, Some(""), Some(""), Some("/home/me"));
        assert_eq!(p, PathBuf::from("/home/me/.local/share/tsuji"));
    }

    #[test]
    fn channel_path_appends_jsonl_extension() {
        let p = channel_path(Path::new("/root"), "default");
        assert_eq!(p, PathBuf::from("/root/default.jsonl"));
    }
}
