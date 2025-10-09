use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use std::path::PathBuf;
use std::fs;
use walkdir::WalkDir;
use rodio::{Decoder, OutputStream, Sink};
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread;
use rand::Rng;

use crate::app::{App, Quadrant};
use crate::theme::DraculaTheme;

#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackMode {
    TrackList,   // Play tracks in order
    Random,      // Play tracks randomly
    Repeat,      // Repeat the entire playlist
    CurrentOnly, // Repeat only current track
}

impl PlaybackMode {
    pub fn next(&self) -> Self {
        match self {
            PlaybackMode::TrackList => PlaybackMode::Random,
            PlaybackMode::Random => PlaybackMode::Repeat,
            PlaybackMode::Repeat => PlaybackMode::CurrentOnly,
            PlaybackMode::CurrentOnly => PlaybackMode::TrackList,
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            PlaybackMode::TrackList => "Track List",
            PlaybackMode::Random => "Random",
            PlaybackMode::Repeat => "Repeat",
            PlaybackMode::CurrentOnly => "Current Only",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            PlaybackMode::TrackList => "📄",
            PlaybackMode::Random => "🔀",
            PlaybackMode::Repeat => "🔁",
            PlaybackMode::CurrentOnly => "🔂",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Track {
    pub name: String,
    pub path: PathBuf,
    pub duration: Option<String>,
}

pub struct TrackList {
    pub tracks: Vec<Track>,
    pub current_track: Option<usize>,
    pub selected_index: usize,
    pub list_state: ListState,
    pub music_folder: PathBuf,
    pub sink: Option<Arc<Mutex<Sink>>>,
    pub _stream: Option<OutputStream>,
    pub is_playing: bool,
    pub is_paused: bool,
    pub playback_mode: PlaybackMode,
}

impl TrackList {

    pub fn new(music_directory: Option<&str>) -> Self {
        let music_folder = if let Some(dir) = music_directory {
            // Expand ~ to home directory if present
            if dir.starts_with("~/") {
                if let Some(home) = dirs::home_dir() {
                    home.join(&dir[2..])
                } else {
                    PathBuf::from(dir)
                }
            } else {
                PathBuf::from(dir)
            }
        } else {
            // Use default logic if no config provided
            dirs::audio_dir()
                .or_else(|| dirs::home_dir().map(|p| p.join("Music")))
                .unwrap_or_else(|| PathBuf::from("./music"))
        };

        let mut track_list = Self {
            tracks: Vec::new(),
            current_track: None,
            selected_index: 0,
            list_state: ListState::default(),
            music_folder,
            sink: None,
            _stream: None,
            is_playing: false,
            is_paused: false,
            playback_mode: PlaybackMode::TrackList,
        };

        track_list.load_tracks();
        track_list.list_state.select(Some(0));
        track_list
    }

    pub fn load_tracks(&mut self) {
        self.tracks.clear();
        
        if !self.music_folder.exists() {
            // Create a default music folder and add some sample entries
            let _ = fs::create_dir_all(&self.music_folder);
            self.tracks.push(Track {
                name: "No music files found".to_string(),
                path: PathBuf::new(),
                duration: None,
            });
            self.tracks.push(Track {
                name: format!("Looking in: {}", self.music_folder.display()),
                path: PathBuf::new(),
                duration: None,
            });
            return;
        }

        // Supported audio formats
        let audio_extensions = vec!["mp3", "wav", "flac", "m4a", "aac", "ogg"];

        for entry in WalkDir::new(&self.music_folder)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if let Some(extension) = entry.path().extension() {
                if audio_extensions.contains(&extension.to_string_lossy().to_lowercase().as_str()) {
                    let name = entry.path()
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Unknown")
                        .to_string();

                    self.tracks.push(Track {
                        name,
                        path: entry.path().to_path_buf(),
                        duration: None, // TODO: Could extract duration with metadata
                    });
                }
            }
        }

        if self.tracks.is_empty() {
            self.tracks.push(Track {
                name: "No audio files found".to_string(),
                path: PathBuf::new(),
                duration: None,
            });
            self.tracks.push(Track {
                name: format!("Searched in: {}", self.music_folder.display()),
                path: PathBuf::new(),
                duration: None,
            });
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, app: &App) {
        let is_focused = app.focused_quadrant == Quadrant::BottomRight;
        
        let status = if self.is_playing && !self.is_paused {
            "▶ Playing"
        } else if self.is_paused {
            "⏸ Paused"
        } else {
            "⏹ Stopped"
        };

        let items: Vec<ListItem> = self.tracks
            .iter()
            .enumerate()
            .map(|(i, track)| {
                let prefix = if Some(i) == self.current_track {
                    if self.is_playing && !self.is_paused {
                        "▶ "
                    } else if self.is_paused {
                        "⏸ "
                    } else {
                        "● "
                    }
                } else {
                    "  "
                };
                
                ListItem::new(format!("{}{}", prefix, track.name))
                    .style(if Some(i) == self.current_track {
                        Style::default().fg(DraculaTheme::GREEN)
                    } else {
                        Style::default().fg(DraculaTheme::FOREGROUND)
                    })
            })
            .collect();

        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .fg(DraculaTheme::BACKGROUND)
                    .bg(DraculaTheme::PURPLE)
            )
            .highlight_symbol("► ");

        let title = format!("🎵 Music Player - {} | {} {}", 
                            status, 
                            self.playback_mode.icon(), 
                            self.playback_mode.to_string());

        let block = if is_focused {
            Block::default()
                .borders(Borders::ALL)
                .title(title.as_str())
                .title_style(Style::default().fg(DraculaTheme::YELLOW))
                .border_style(Style::default().fg(DraculaTheme::PINK))
        } else {
            Block::default()
                .borders(Borders::ALL)
                .title(title.as_str())
                .title_style(Style::default().fg(DraculaTheme::YELLOW))
                .border_style(Style::default().fg(DraculaTheme::COMMENT))
        };

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Use the full inner area for the track list
        frame.render_stateful_widget(list, inner, &mut self.list_state);
    }

    pub fn move_selection_up(&mut self) {
        if !self.tracks.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.tracks.len() - 1
            } else {
                self.selected_index - 1
            };
            self.list_state.select(Some(self.selected_index));
        }
    }

    pub fn move_selection_down(&mut self) {
        if !self.tracks.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.tracks.len();
            self.list_state.select(Some(self.selected_index));
        }
    }

    pub fn play_selected(&mut self) {
        if self.selected_index < self.tracks.len() {
            self.play_track(self.selected_index);
        }
    }

    pub fn play_track(&mut self, index: usize) {
        if index >= self.tracks.len() {
            return;
        }

        let track_path = self.tracks[index].path.clone();
        if !track_path.exists() {
            return;
        }

        // Stop current playback
        self.stop();

        // Initialize audio stream if needed
        if self.sink.is_none() {
            if let Ok((stream, stream_handle)) = OutputStream::try_default() {
                if let Ok(sink) = Sink::try_new(&stream_handle) {
                    self.sink = Some(Arc::new(Mutex::new(sink)));
                    self._stream = Some(stream);
                }
            }
        }

        if let Some(sink_arc) = &self.sink {
            let sink_clone = Arc::clone(sink_arc);
            
            thread::spawn(move || {
                if let Ok(file) = fs::File::open(&track_path) {
                    if let Ok(source) = Decoder::new(BufReader::new(file)) {
                        if let Ok(sink) = sink_clone.lock() {
                            sink.append(source);
                            sink.play();
                        }
                    }
                }
            });

            self.current_track = Some(index);
            self.is_playing = true;
            self.is_paused = false;
        }
    }

    pub fn toggle_play_pause(&mut self) {
        if let Some(sink_arc) = &self.sink {
            let mut should_play_selected = false;
            let mut should_play_current = false;
            
            {
                if let Ok(sink) = sink_arc.lock() {
                    if self.is_playing && !self.is_paused {
                        sink.pause();
                        self.is_paused = true;
                        return;
                    } else if self.is_paused {
                        sink.play();
                        self.is_paused = false;
                        return;
                    }
                    
                    should_play_current = self.current_track.is_some();
                    should_play_selected = !should_play_current;
                }
            }
            
            if should_play_current {
                if let Some(current) = self.current_track {
                    self.play_track(current);
                }
            } else if should_play_selected {
                self.play_selected();
            }
        } else {
            self.play_selected();
        }
    }

    pub fn stop(&mut self) {
        if let Some(sink_arc) = &self.sink {
            if let Ok(sink) = sink_arc.lock() {
                sink.stop();
            }
        }
        self.is_playing = false;
        self.is_paused = false;
    }

    pub fn next_track(&mut self) {
        if !self.tracks.is_empty() {
            let next_index = self.current_track
                .map(|i| (i + 1) % self.tracks.len())
                .unwrap_or(0);
            self.play_track(next_index);
        }
    }

    pub fn previous_track(&mut self) {
        if !self.tracks.is_empty() {
            let prev_index = self.current_track
                .map(|i| if i == 0 { self.tracks.len() - 1 } else { i - 1 })
                .unwrap_or(0);
            self.play_track(prev_index);
        }
    }

    pub fn cycle_playback_mode(&mut self) {
        self.playback_mode = self.playback_mode.next();
    }

    pub fn refresh_library(&mut self) {
        self.stop();
        self.load_tracks();
        self.selected_index = 0;
        self.list_state.select(Some(0));
        self.current_track = None;
    }

    /// Update the music directory and reload tracks
    pub fn update_music_directory(&mut self, music_directory: Option<&str>) {
        let new_folder = if let Some(dir) = music_directory {
            // Expand ~ to home directory if present
            if dir.starts_with("~/") {
                if let Some(home) = dirs::home_dir() {
                    home.join(&dir[2..])
                } else {
                    PathBuf::from(dir)
                }
            } else {
                PathBuf::from(dir)
            }
        } else {
            // Use default logic if no config provided
            dirs::audio_dir()
                .or_else(|| dirs::home_dir().map(|p| p.join("Music")))
                .unwrap_or_else(|| PathBuf::from("./music"))
        };

        self.music_folder = new_folder;
        self.refresh_library();
    }

    /// Check if current track has finished and handle auto-advance
    pub fn update_playback_state(&mut self) {
        let should_advance = if let Some(sink_arc) = &self.sink {
            if let Ok(sink) = sink_arc.lock() {
                // Check if the sink is empty (track finished) and we were playing
                sink.empty() && self.is_playing && !self.is_paused
            } else {
                false
            }
        } else {
            false
        };

        if should_advance {
            // Track has finished, handle auto-advance based on playback mode
            self.handle_track_finished();
        }
    }

    /// Temporarily lower the music volume during alarm
    pub fn lower_volume_for_alarm(&mut self, alarm_volume: f32) {
        if let Some(sink_arc) = &self.sink {
            if let Ok(sink) = sink_arc.lock() {
                sink.set_volume(alarm_volume);
            }
        }
    }

    /// Restore the normal music volume after alarm
    pub fn restore_volume(&mut self, normal_volume: f32) {
        if let Some(sink_arc) = &self.sink {
            if let Ok(sink) = sink_arc.lock() {
                sink.set_volume(normal_volume);
            }
        }
    }

    /// Handle what happens when a track finishes playing
    fn handle_track_finished(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        match self.playback_mode {
            PlaybackMode::TrackList => {
                // Play next track in order, stop at the end
                if let Some(current) = self.current_track {
                    let next_index = current + 1;
                    if next_index < self.tracks.len() {
                        self.play_track(next_index);
                    } else {
                        // Reached the end of the playlist
                        self.stop();
                    }
                }
            }
            PlaybackMode::Random => {
                // Play a random track
                self.play_random_track();
            }
            PlaybackMode::Repeat => {
                // Play next track in order, loop back to beginning
                if let Some(current) = self.current_track {
                    let next_index = (current + 1) % self.tracks.len();
                    self.play_track(next_index);
                } else {
                    self.play_track(0);
                }
            }
            PlaybackMode::CurrentOnly => {
                // Repeat the same track
                if let Some(current) = self.current_track {
                    self.play_track(current);
                }
            }
        }
    }

    /// Play a random track from the playlist
    fn play_random_track(&mut self) {
        if !self.tracks.is_empty() {
            let mut rng = rand::thread_rng();
            let random_index = rng.gen_range(0..self.tracks.len());
            self.play_track(random_index);
        }
    }

    // Legacy methods for compatibility
    pub fn add_track(&mut self, track: String) {
        // This is now handled by load_tracks() from filesystem
        let _ = track; // Suppress unused parameter warning
    }
}