use std::{
    fs::{self, File},
    io::{self, Read},
    process::Command,
};

use crate::themes_list::THEME_LIST;
use clap::Subcommand;
use std::io::Write;

#[derive(Subcommand)]
pub enum ThemeActions {
    ///Install theme by <NAME>
    #[clap()]
    Install { name: String },

    /// List all available Zola Themes
    #[clap()]
    List,
}

pub fn handle_command_line_action(action: ThemeActions) {
    match action {
        ThemeActions::Install { name } => install_theme(name),
        ThemeActions::List => list_themes(),
    }
}

const THEME_PROPERTY: &str = "theme";

fn clone_repo(url: &str, name: &str) -> bool {
    let path = format!("themes\\{name}");
    if fs::exists(&path).unwrap_or(false) {
        return true;
    }

    match Command::new("git").args(["clone", url, &path]).output() {
        Err(err) => {
            console::error(&format!("Failed to clone the repo '{url}: [{err}]"));
            console::info("help: Make sure that git client is well installed..");

            false
        }
        Ok(status) if !status.status.success() => {
            console::error(&format!(
                "Failed to clone the repo '{url}': [{}]",
                status.status.code().unwrap()
            ));
            false
        }
        _ => true,
    }
}

fn add_theme_line(buffer: &mut String, name: &str) {
    buffer.push_str(THEME_PROPERTY);
    buffer.push_str(" = ");
    buffer.push_str(&libs::toml::Value::String(name.to_string()).to_string());
    buffer.push('\n');
}

fn install_theme(name: String) {
    console::info(&format!("Searching theme {name} ..."));

    let Some(entry) = THEME_LIST.iter().find(|(k, _)| k.to_lowercase() == name.to_lowercase())
    else {
        console::error(&format!("Theme {name} is not found"));
        return;
    };

    console::info("Checking config.toml");
    let Ok(mut config_file) = File::open("config.toml") else {
        console::error("This directory does not have 'config.toml'");
        return;
    };

    let mut content = String::new();
    if config_file.read_to_string(&mut content).is_err() {
        console::error("failed to read 'config.toml' content");
        return;
    }

    let (theme_name, theme_repo) = entry;
    console::info(&format!("Downloading theme {theme_name}... "));
    if !clone_repo(theme_repo, theme_name) {
        return;
    }

    console::info("Updating config.toml... ");
    let output = change_theme_in_config(theme_name, content);

    drop(config_file);
    let _ = fs::rename("config.toml", "config.toml.bak");
    let Ok(mut config_file) = File::create("config.toml") else {
        console::error("This directory does not have 'config.toml'");
        return;
    };

    if let Err(err) = write!(config_file, "{}", output) {
        console::error(&format!("Failed to write the config file content :[{err}]"));
        return;
    }
    console::success(
        "Theme is successfully installed, please check the documentation for further instructions",
    );
}

fn change_theme_in_config(theme_name: &str, content: String) -> String {
    let mut output = String::with_capacity(content.len() + 128);
    let mut is_added = false;
    for line in content.lines() {
        if !is_added && line.trim().to_lowercase().starts_with(THEME_PROPERTY) {
            add_theme_line(&mut output, theme_name);
            output.push('\n');
            is_added = true;
        } else if !is_added && line.trim_start().to_lowercase().starts_with('[') {
            add_theme_line(&mut output, theme_name);
            output.push_str(line);
            output.push('\n');
            is_added = true;
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }

    if !is_added {
        add_theme_line(&mut output, theme_name);
        output.push('\n');
    }
    output
}

pub fn list_themes() {
    for entry in THEME_LIST {
        console::info(entry.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_theme_line() {
        let mut theme = String::new();
        add_theme_line(&mut theme, "abc");
        assert_eq!(theme, "theme = \"abc\"\n");
        theme.clear();
        add_theme_line(&mut theme, "\"abc\"");
        assert_eq!(theme, "theme = \'\"abc\"\'\n");
    }
}
