use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use color_eyre::Result;

/// Configuration for the sessio application
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// Timer configuration
    pub timer: TimerConfig,
    /// Summary configuration
    pub summary: SummaryConfig,
    /// Todo configuration 
    pub todo: TodoConfig,
    /// Music/Track configuration
    pub music: MusicConfig,
    /// Theme configuration
    pub theme: ThemeConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimerConfig {
    /// Work session duration in minutes (default: 25)
    pub work_minutes: u64,
    /// Short break duration in minutes (default: 5)
    pub short_break_minutes: u64,
    /// Long break duration in minutes (default: 15)
    pub long_break_minutes: u64,
    /// Number of work sessions before long break (default: 4)
    pub sessions_until_long_break: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SummaryConfig {
    /// Show summary at the end of each pomodoro (default: true)
    pub daily_goal_minutes: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TodoConfig {
    /// Auto-save todos to file (default: true)
    pub auto_save: bool,
    /// Path to save todos (default: ~/.config/sessio/todos.json)
    pub save_path: Option<String>,
    /// Save pomodoro session data (default: true)
    #[serde(default = "default_save_pomodoro_data")]
    pub save_pomodoro_data: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MusicConfig {
    /// Default music directory to scan for tracks
    pub music_directory: Option<String>,
    /// Default volume (0.0 to 1.0, default: 0.7)
    pub default_volume: f32,
    /// Auto-play next track (default: true)
    pub auto_play_next: bool,
    /// Volume during alarm (0.0 to 1.0, default: 0.3)
    pub alarm_volume: f32,
    /// Alarm duration in seconds (default: 15)
    pub alarm_duration_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeConfig {
    /// Use Dracula theme (default: true)
    pub use_dracula: bool,
}

// Default functions for serde
fn default_save_pomodoro_data() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Config {
            timer: TimerConfig::default(),
            summary: SummaryConfig::default(),
            todo: TodoConfig::default(),
            music: MusicConfig::default(),
            theme: ThemeConfig::default(),
        }
    }
}

impl Default for TimerConfig {
    fn default() -> Self {
        TimerConfig {
            work_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
            sessions_until_long_break: 4,
        }
    }
}

impl Default for SummaryConfig {
    fn default() -> Self {
        SummaryConfig {
            daily_goal_minutes: 120,
        }
    }
}

impl Default for TodoConfig {
    fn default() -> Self {
        TodoConfig {
            auto_save: true,
            save_path: Some("~/.config/sessio/todos.md".to_string()),
            save_pomodoro_data: true,
        }
    }
}

impl Default for MusicConfig {
    fn default() -> Self {
        MusicConfig {
            music_directory: Some("~/Music".to_string()),
            default_volume: 0.7,
            auto_play_next: true,
            alarm_volume: 0.3,
            alarm_duration_seconds: 15,
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        ThemeConfig {
            use_dracula: true,
        }
    }
}

impl Config {
    /// Get the default config file path: ~/.config/sessio/sessio.toml
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not find config directory"))?;
        
        let sessio_config_dir = config_dir.join("sessio");
        
        // Create the config directory if it doesn't exist
        if !sessio_config_dir.exists() {
            fs::create_dir_all(&sessio_config_dir)?;
        }
        
        Ok(sessio_config_dir.join("sessio.toml"))
    }
    
    /// Load configuration from file, creating default if it doesn't exist
    pub fn load() -> Result<Config> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let config_content = fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&config_content)?;
            Ok(config)
        } else {
            // Create default config and save it
            let default_config = Config::default();
            default_config.save()?;
            Ok(default_config)
        }
    }
    
    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        // Generate a nicely formatted config file with comments (like the example)
        let config_content = self.to_formatted_toml();
        fs::write(&config_path, config_content)?;
        Ok(())
    }
    
    /// Generate a formatted TOML string with comments
    fn to_formatted_toml(&self) -> String {
        format!(
            r#"# sessio Configuration File
# This file is located at ~/.config/sessio/sessio.toml
#
# The application will automatically create this configuration file with default values
# if one doesn't exist. You can modify these settings and reload with 'C' key in the app.

[timer]
# Pomodoro timer settings (current values shown)
work_minutes = {}                    # Duration of work sessions in minutes
short_break_minutes = {}             # Duration of short breaks in minutes
long_break_minutes = {}              # Duration of long breaks in minutes
sessions_until_long_break = {}       # Number of work sessions before a long break

[summary]
# Summary panel settings (current values shown)
daily_goal_minutes = {}              # Daily focus time goal in minutes

[todo]
# Todo list settings (current values shown)
auto_save = {}                       # Automatically save todos to file
save_pomodoro_data = {}             # Save pomodoro session data to todos.md
{}

[music]
# Music player settings (current values shown)
{}default_volume = {}                # Default volume (0.0 to 1.0)
auto_play_next = {}                  # Automatically play next track when current ends
alarm_volume = {}                    # Volume during alarm notification (0.0 to 1.0)
alarm_duration_seconds = {}          # How long the alarm sound lasts in seconds

[theme]
# Theme settings (current values shown)
use_dracula = {}                     # Use the Dracula color theme

# Configuration can be reloaded at runtime by pressing 'C' (capital C) in the application
"#,
            self.timer.work_minutes,
            self.timer.short_break_minutes,
            self.timer.long_break_minutes,
            self.timer.sessions_until_long_break,
            self.summary.daily_goal_minutes,
            self.todo.auto_save,
            self.todo.save_pomodoro_data,
            if let Some(ref path) = self.todo.save_path {
                format!("save_path = \"{}\"                   # Custom path for saving todos\n", path)
            } else {
                "# save_path = \"custom/path/todos.json\"  # Optional: custom path for saving todos\n".to_string()
            },
            if let Some(ref dir) = self.music.music_directory {
                format!("music_directory = \"{}\"           # Directory to scan for music files\n", dir)
            } else {
                "# music_directory = \"/path/to/music\"   # Optional: directory to scan for music files\n".to_string()
            },
            self.music.default_volume,
            self.music.auto_play_next,
            self.music.alarm_volume,
            self.music.alarm_duration_seconds,
            self.theme.use_dracula
        )
    }
    
    /// Reload configuration from file
    pub fn reload(&mut self) -> Result<()> {
        let new_config = Self::load()?;
        *self = new_config;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.timer.work_minutes, 25);
        assert_eq!(config.timer.short_break_minutes, 5);
        assert_eq!(config.timer.long_break_minutes, 15);
        assert_eq!(config.timer.sessions_until_long_break, 4);
        assert!(config.todo.auto_save);
        assert_eq!(config.music.default_volume, 0.7);
        assert!(config.music.auto_play_next);
        assert!(config.theme.use_dracula);
    }
    
    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let serialized = toml::to_string_pretty(&config).expect("Failed to serialize config");
        let deserialized: Config = toml::from_str(&serialized).expect("Failed to deserialize config");
        
        assert_eq!(config.timer.work_minutes, deserialized.timer.work_minutes);
        assert_eq!(config.todo.auto_save, deserialized.todo.auto_save);
    }
}