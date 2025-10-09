use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};
use std::time::{Duration, Instant};
use rodio::{OutputStream, Sink, Decoder};
use std::thread;
use std::fs::File;
use std::io::BufReader;
use chrono::{DateTime, Local, NaiveDate};
use std::sync::{Arc, Mutex};

use crate::app::{App, Quadrant};
use crate::theme::DraculaTheme;
use crate::todo::TodoItem;
use crate::config::Config;

// Helper function to format duration
fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

#[derive(Debug, Clone, PartialEq)]
pub enum PomodoroPhase {
    Work,
    ShortBreak,
    LongBreak,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimerState {
    Stopped,
    Running,
    Paused,
}

#[derive(Debug, Clone)]
pub struct PomodoroSession {
    pub date: chrono::NaiveDate,
    pub work_sessions: u32,
    pub total_work_minutes: u32,
    pub break_sessions: u32,
    pub total_break_minutes: u32,
    pub tasks_worked_on: Vec<String>, // Task names that were worked on
}

pub struct Timer {
    pub state: TimerState,
    pub phase: PomodoroPhase,
    pub pomodoro_count: u32,
    pub time_remaining: Duration,
    pub last_tick: Option<Instant>,
    pub selected_todo_index: Option<usize>, // Track which TODO item is being timed
    pub work_completed_flag: bool, // Flag to track when work session completes
    
    // Pomodoro durations (in seconds)
    pub work_duration: Duration,
    pub short_break_duration: Duration,
    pub long_break_duration: Duration,
    pub long_break_interval: u32, // Every N pomodoros
    
    // Daily session tracking
    pub daily_sessions: Vec<PomodoroSession>,
    pub current_session_start: Option<chrono::DateTime<chrono::Local>>,
    
    // Alarm settings
    pub alarm_volume: f32,
    pub alarm_duration_seconds: u64,
    pub alarm_active: bool,
    pub alarm_end_time: Option<Instant>,
}

impl Timer {
    pub fn new(work_minutes: u64, short_break_minutes: u64, long_break_minutes: u64, sessions_until_long_break: u32, alarm_volume: f32, alarm_duration_seconds: u64) -> Self {
        Self {
            state: TimerState::Stopped,
            phase: PomodoroPhase::Work,
            pomodoro_count: 0,
            time_remaining: Duration::from_secs(work_minutes * 60), // Convert minutes to seconds
            last_tick: None,
            selected_todo_index: None,
            work_completed_flag: false,
            work_duration: Duration::from_secs(work_minutes * 60),        // Work duration
            short_break_duration: Duration::from_secs(short_break_minutes * 60),   // Short break duration
            long_break_duration: Duration::from_secs(long_break_minutes * 60),   // Long break duration
            long_break_interval: sessions_until_long_break, // Long break every N pomodoros
            daily_sessions: Vec::new(),
            current_session_start: None,
            alarm_volume,
            alarm_duration_seconds,
            alarm_active: false,
            alarm_end_time: None,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, app: &App, todo_items: &[TodoItem]) {
        // Update timer if running
        if self.state == TimerState::Running {
            self.update();
        }
        
        let is_focused = app.focused_quadrant == Quadrant::TopLeft;
        
        // Create layout within the timer panel for content and progress bar
        let inner_area = if is_focused {
            Block::default()
                .borders(Borders::ALL)
                .title("⏱️  Pomodoro Timer")
                .border_style(Style::default().fg(DraculaTheme::PINK))
                .inner(area)
        } else {
            Block::default()
                .borders(Borders::ALL)
                .title("⏱️  Pomodoro Timer")
                .border_style(Style::default().fg(DraculaTheme::COMMENT))
                .inner(area)
        };
        
        let timer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(6),      // Main content
                Constraint::Length(1),   // Progress bar (no borders, just the bar)
            ])
            .split(inner_area);
        
        // Format time remaining
        let total_secs = self.time_remaining.as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        let time_display = format!("{:02}:{:02}", minutes, seconds);
        
        // Calculate progress percentage
        let total_duration = match self.phase {
            PomodoroPhase::Work => self.work_duration,
            PomodoroPhase::ShortBreak => self.short_break_duration,
            PomodoroPhase::LongBreak => self.long_break_duration,
        };
        let elapsed = total_duration.saturating_sub(self.time_remaining);
        let progress_ratio = if total_duration.as_secs() > 0 {
            (elapsed.as_secs() as f64 / total_duration.as_secs() as f64 * 100.0) as u16
        } else {
            0
        };
        
        // Get phase info
        let (phase_name, phase_emoji, phase_color) = match self.phase {
            PomodoroPhase::Work => ("WORK", "🍅", DraculaTheme::RED),
            PomodoroPhase::ShortBreak => ("SHORT BREAK", "☕", DraculaTheme::GREEN),
            PomodoroPhase::LongBreak => ("LONG BREAK", "🌴", DraculaTheme::CYAN),
        };
        
        // Get state info
        let (state_text, _state_color) = match self.state {
            TimerState::Stopped => ("Ready", DraculaTheme::COMMENT),
            TimerState::Running => ("Running", DraculaTheme::GREEN),
            TimerState::Paused => ("Paused", DraculaTheme::YELLOW),
        };
        
        // Get selected task info
        let selected_task_info = if let Some(index) = self.selected_todo_index {
            if let Some(task) = todo_items.get(index) {
                format!("\n🎯 Working on: {}", 
                    if task.task.len() > 30 { 
                        format!("{}...", &task.task[..30]) 
                    } else { 
                        task.task.clone() 
                    }
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        
        let content = format!(
            "{} {} Phase\nPomodoros completed: {}\n\n⏱️  {}\nStatus: {}{}",
            phase_emoji,
            phase_name,
            self.pomodoro_count,
            time_display,
            state_text,
            selected_task_info
        );
        
        // Render the main timer border first
        let timer_block = if is_focused {
            Block::default()
                .borders(Borders::ALL)
                .title("⏱️  Pomodoro Timer")
                .title_style(Style::default().fg(phase_color))
                .border_style(Style::default().fg(DraculaTheme::PINK))
                .style(Style::default().bg(DraculaTheme::BACKGROUND))
        } else {
            Block::default()
                .borders(Borders::ALL)
                .title("⏱️  Pomodoro Timer")
                .title_style(Style::default().fg(phase_color))
                .border_style(Style::default().fg(DraculaTheme::COMMENT))
                .style(Style::default().bg(DraculaTheme::BACKGROUND))
        };
        
        frame.render_widget(timer_block, area);
        
        // Render main timer content
        let timer_content = Paragraph::new(content)
            .style(Style::default().fg(DraculaTheme::FOREGROUND).bg(DraculaTheme::BACKGROUND));
        
        frame.render_widget(timer_content, timer_layout[0]);

        // Create progress bar (no border, just the bar)
        let progress_label = format!("{}% - {} elapsed", progress_ratio, format_duration(elapsed));
        let progress_bar = Gauge::default()
            .gauge_style(Style::default().fg(phase_color).bg(DraculaTheme::CURRENT_LINE))
            .percent(progress_ratio)
            .label(progress_label)
            .style(Style::default().fg(DraculaTheme::FOREGROUND));

        frame.render_widget(progress_bar, timer_layout[1]);
    }

    // Timer functionality methods
    pub fn update(&mut self) {
        if self.state != TimerState::Running {
            return;
        }
        
        let now = Instant::now();
        if let Some(last_tick) = self.last_tick {
            let elapsed = now.duration_since(last_tick);
            if elapsed >= self.time_remaining {
                // Timer finished
                self.time_remaining = Duration::ZERO;
                self.complete_phase();
            } else {
                self.time_remaining -= elapsed;
            }
        }
        self.last_tick = Some(now);
    }
    
    fn complete_phase(&mut self) {
        // Play alarm sound when any phase completes
        self.play_alarm();
        
        match self.phase {
            PomodoroPhase::Work => {
                // Record work session completion
                let work_minutes = (self.work_duration.as_secs() / 60) as u32;
                {
                    let today_session = self.get_today_session();
                    today_session.work_sessions += 1;
                    today_session.total_work_minutes += work_minutes;
                }
                
                // Set the flag when work completes and we have a selected TODO
                if self.selected_todo_index.is_some() {
                    self.work_completed_flag = true;
                }
                
                self.pomodoro_count += 1;
                // Clear session start time
                self.current_session_start = None;
                
                // Decide next break type
                if self.pomodoro_count % self.long_break_interval == 0 {
                    self.phase = PomodoroPhase::LongBreak;
                    self.time_remaining = self.long_break_duration;
                } else {
                    self.phase = PomodoroPhase::ShortBreak;
                    self.time_remaining = self.short_break_duration;
                }
            }
            PomodoroPhase::ShortBreak => {
                // Record break completion
                let break_minutes = (self.short_break_duration.as_secs() / 60) as u32;
                {
                    let today_session = self.get_today_session();
                    today_session.break_sessions += 1;
                    today_session.total_break_minutes += break_minutes;
                }
                
                self.phase = PomodoroPhase::Work;
                self.time_remaining = self.work_duration;
            }
            PomodoroPhase::LongBreak => {
                // Record long break completion
                let break_minutes = (self.long_break_duration.as_secs() / 60) as u32;
                {
                    let today_session = self.get_today_session();
                    today_session.break_sessions += 1;
                    today_session.total_break_minutes += break_minutes;
                }
                
                self.phase = PomodoroPhase::Work;
                self.time_remaining = self.work_duration;
            }
        }
        self.state = TimerState::Stopped;
        self.last_tick = None;
    }

    /// Play an alarm sound when timer completes
    /// Sets the alarm state for coordinating with music volume
    fn play_alarm(&mut self) {
        let alarm_volume = self.alarm_volume;
        let alarm_duration = self.alarm_duration_seconds;
        
        // Set alarm state
        self.alarm_active = true;
        self.alarm_end_time = Some(Instant::now() + Duration::from_secs(alarm_duration));
        
        // Spawn a thread to play the alarm sound without blocking
        thread::spawn(move || {
            // Try to load alarm sound from config directory
            let alarm_path = if let Some(config_dir) = dirs::config_dir() {
                let sessio_config_dir = config_dir.join("sessio");
                let alarm_file = sessio_config_dir.join("alarm.wav");
                if alarm_file.exists() {
                    Some(alarm_file)
                } else {
                    // Try other common audio formats
                    let extensions = ["alarm.mp3", "alarm.ogg", "alarm.flac", "alarm.m4a"];
                    extensions.iter()
                        .map(|ext| sessio_config_dir.join(ext))
                        .find(|path| path.exists())
                }
            } else {
                None
            };

            if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
                if let Ok(sink) = Sink::try_new(&stream_handle) {
                    // Set alarm volume
                    sink.set_volume(alarm_volume);
                    
                    if let Some(path) = alarm_path {
                        // Play the audio file
                        if let Ok(file) = File::open(&path) {
                            let buf_reader = BufReader::new(file);
                            if let Ok(source) = Decoder::new(buf_reader) {
                                sink.append(source);
                                
                                // Wait for the specified alarm duration
                                let start_time = std::time::Instant::now();
                                while !sink.empty() && start_time.elapsed().as_secs() < alarm_duration {
                                    std::thread::sleep(std::time::Duration::from_millis(100));
                                }
                                
                                // Stop the alarm after the duration
                                sink.stop();
                                return;
                            }
                        }
                    }
                    
                    // Fallback: create a simple beep tone for the duration if no audio file found
                    let beep_count = (alarm_duration as f32 / 0.5).ceil() as u64; // Beep every 500ms
                    for _ in 0..beep_count {
                        print!("\x07"); // ASCII bell character
                        std::io::Write::flush(&mut std::io::stdout()).ok();
                        std::thread::sleep(std::time::Duration::from_millis(500));
                    }
                }
            }
        });
    }

    pub fn start(&mut self) {
        match self.state {
            TimerState::Stopped | TimerState::Paused => {
                self.state = TimerState::Running;
                self.last_tick = Some(Instant::now());
                
                // Record session start time for work phases
                if self.phase == PomodoroPhase::Work && self.current_session_start.is_none() {
                    self.current_session_start = Some(chrono::Local::now());
                }
            }
            TimerState::Running => {
                // Pause
                self.state = TimerState::Paused;
                self.last_tick = None;
            }
        }
    }

    pub fn stop(&mut self) {
        self.state = TimerState::Stopped;
        self.last_tick = None;
    }

    pub fn reset(&mut self) {
        self.state = TimerState::Stopped;
        self.last_tick = None;
        self.time_remaining = match self.phase {
            PomodoroPhase::Work => self.work_duration,
            PomodoroPhase::ShortBreak => self.short_break_duration,
            PomodoroPhase::LongBreak => self.long_break_duration,
        };
    }
    
    pub fn skip_phase(&mut self) {
        self.complete_phase();
    }
    
    pub fn toggle_start_pause(&mut self) {
        self.start(); // start() already handles the toggle logic
    }
    
    pub fn set_selected_todo(&mut self, index: Option<usize>) {
        self.selected_todo_index = index;
    }
    
    pub fn set_selected_todo_with_task_name(&mut self, index: Option<usize>, task_name: Option<String>) {
        self.selected_todo_index = index;
        
        // Add task name to today's session if provided
        if let Some(name) = task_name {
            let today_session = self.get_today_session();
            if !today_session.tasks_worked_on.contains(&name) {
                today_session.tasks_worked_on.push(name);
            }
        }
    }
    
    pub fn get_selected_todo(&self) -> Option<usize> {
        self.selected_todo_index
    }
    
    // Returns the time that should be added to the TODO item when work phase completes
    // Returns the work duration in minutes
    pub fn get_work_session_minutes(&self) -> u32 {
        (self.work_duration.as_secs() / 60) as u32
    }
    
    // Check if a work phase just completed (to add time to TODO)
    pub fn work_phase_just_completed(&self) -> bool {
        self.work_completed_flag && self.selected_todo_index.is_some()
    }
    
    // Clear the work completed flag after processing
    pub fn clear_work_completed_flag(&mut self) {
        self.work_completed_flag = false;
    }
    
    // Session tracking methods
    pub fn get_today_session(&mut self) -> &mut PomodoroSession {
        let today = chrono::Local::now().date_naive();
        
        // Check if we already have a session for today
        let session_exists = self.daily_sessions.iter().any(|s| s.date == today);
        
        if !session_exists {
            // Create a new session for today
            self.daily_sessions.push(PomodoroSession {
                date: today,
                work_sessions: 0,
                total_work_minutes: 0,
                break_sessions: 0,
                total_break_minutes: 0,
                tasks_worked_on: Vec::new(),
            });
        }
        
        self.daily_sessions.iter_mut().find(|s| s.date == today).unwrap()
    }
    
    pub fn get_daily_sessions(&self) -> &[PomodoroSession] {
        &self.daily_sessions
    }
    
    pub fn load_daily_sessions(&mut self, sessions: Vec<PomodoroSession>) {
        self.daily_sessions = sessions;
        
        // Restore today's pomodoro count from the loaded sessions
        let today = chrono::Local::now().date_naive();
        if let Some(today_session) = self.daily_sessions.iter().find(|s| s.date == today) {
            self.pomodoro_count = today_session.work_sessions;
        }
    }
    
    /// Update alarm state and return true if alarm should still be active
    pub fn update_alarm_state(&mut self) -> bool {
        if self.alarm_active {
            if let Some(end_time) = self.alarm_end_time {
                if Instant::now() >= end_time {
                    self.alarm_active = false;
                    self.alarm_end_time = None;
                    return false;
                }
                return true;
            }
        }
        false
    }
    
    /// Check if alarm is currently active
    pub fn is_alarm_active(&self) -> bool {
        self.alarm_active
    }
    
    /// Get alarm volume setting
    pub fn get_alarm_volume(&self) -> f32 {
        self.alarm_volume
    }
}