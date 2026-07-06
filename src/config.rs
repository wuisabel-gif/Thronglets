use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct AppConfig {
    pub theme: Option<String>,
}

pub fn config_path() -> PathBuf {
    if let Some(xdg) = nonempty_env("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg).join("thronglets").join("config.toml");
    }
    if let Some(home) = nonempty_env("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("thronglets")
            .join("config.toml");
    }
    PathBuf::from(".config/thronglets/config.toml")
}

pub fn load(path: &PathBuf, warnings: &mut Vec<String>) -> AppConfig {
    let contents = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return AppConfig::default(),
        Err(e) => {
            warnings.push(format!(
                "cannot read config {} ({e}) - using defaults",
                path.display()
            ));
            return AppConfig::default();
        }
    };
    AppConfig {
        theme: parse_string_key(&contents, "theme").or_else(|| {
            warnings.push(format!(
                "config {} has no readable theme setting - using defaults",
                path.display()
            ));
            None
        }),
    }
}

pub fn save_theme(path: &PathBuf, theme: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, format!("theme = \"{}\"\n", escape_toml_string(theme)))
}

fn parse_string_key(contents: &str, key: &str) -> Option<String> {
    for raw_line in contents.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        let Some((left, right)) = line.split_once('=') else {
            continue;
        };
        if left.trim() != key {
            continue;
        }
        let value = right.trim();
        if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
            return Some(value[1..value.len() - 1].replace("\\\"", "\""));
        }
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn nonempty_env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}
