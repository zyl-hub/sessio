# Configuration System

The `sessio` application uses a TOML configuration file located at `~/.config/sessio/sessio.toml`.

## Features

- **Automatic creation**: The config file is created automatically with default values when the application starts if it doesn't exist.
- **Hot reload**: Press `C` (capital C) in the application to reload configuration without restarting.
- **Well-documented**: See `sessio.toml.example` for a complete example with comments.

## Configuration Structure

The configuration is divided into four main sections:

### [timer]
Controls Pomodoro timer behavior:
- `work_minutes`: Duration of work sessions (default: 25)
- `short_break_minutes`: Duration of short breaks (default: 5)  
- `long_break_minutes`: Duration of long breaks (default: 15)
- `sessions_until_long_break`: Work sessions before long break (default: 4)

### [todo]
Controls todo list behavior:
- `max_display_items`: Maximum items shown at once (default: 10)
- `auto_save`: Automatically save todos (default: true)
- `save_path`: Optional custom path for saving todos

### [music]
Controls music player behavior:
- `music_directory`: Optional directory to scan for music files
- `default_volume`: Volume level 0.0-1.0 (default: 0.7)
- `auto_play_next`: Auto-play next track (default: true)

### [theme]
Controls appearance:
- `use_dracula`: Use Dracula color scheme (default: true)

## Usage

1. The application creates `~/.config/sessio/sessio.toml` automatically on first run
2. Edit the file to customize settings
3. Press `C` in the application to reload changes
4. Check `sessio.toml.example` for the complete configuration reference

## Example

```toml
[timer]
work_minutes = 25
short_break_minutes = 5
long_break_minutes = 15
sessions_until_long_break = 4

[todo]
max_display_items = 10
auto_save = true

[music]
default_volume = 0.7
auto_play_next = true

[theme]
use_dracula = true
```