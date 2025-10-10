use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::Block,
    DefaultTerminal, Frame,
};
use std::time::Instant;

mod app;
mod config;
mod theme;
mod timer;
mod summary;
mod todo;
mod track_list;
mod help;

use app::{App, Quadrant};
use config::Config;
use theme::DraculaTheme;
use timer::Timer;
use summary::Summary;
use todo::Todo;
use track_list::TrackList;
use help::Help;

/// Helper function to check if a character is Chinese (CJK)
fn is_chinese_character(c: char) -> bool {
    // Check for Chinese/Japanese/Korean character ranges
    matches!(c as u32,
        0x4E00..=0x9FFF |  // CJK Unified Ideographs
        0x3400..=0x4DBF |  // CJK Extension A
        0x20000..=0x2A6DF | // CJK Extension B
        0x2A700..=0x2B73F | // CJK Extension C
        0x2B740..=0x2B81F | // CJK Extension D
        0x2B820..=0x2CEAF | // CJK Extension E
        0x2CEB0..=0x2EBEF | // CJK Extension F
        0x3000..=0x303F |  // CJK Symbols and Punctuation
        0x3040..=0x309F |  // Hiragana
        0x30A0..=0x30FF |  // Katakana
        0x31F0..=0x31FF |  // Katakana Phonetic Extensions
        0xFF00..=0xFFEF    // Halfwidth and Fullwidth Forms
    )
}

struct AppState {
    app: App,
    timer: Timer,
    summary: Summary,
    todo: Todo,
    track_list: TrackList,
    config: Config,
    last_key_time: Instant,
    last_key_code: Option<KeyCode>,
    was_alarm_active_last_update: bool,
}

impl AppState {
    fn new() -> Result<Self> {
        let config = Config::load()?;
        
        // Extract values to avoid partial moves
        let music_dir = config.music.music_directory.clone();
        let work_minutes = config.timer.work_minutes;
        let short_break_minutes = config.timer.short_break_minutes;
        let long_break_minutes = config.timer.long_break_minutes;
        let sessions_until_long_break = config.timer.sessions_until_long_break;
        let daily_goal_minutes = config.summary.daily_goal_minutes;
        let save_path = config.todo.save_path.clone();
        
        let alarm_volume = config.music.alarm_volume;
        let alarm_duration_seconds = config.music.alarm_duration_seconds;
        let alarm_file_path = config.music.alarm_file_path.clone();
        let mut timer = Timer::new(work_minutes, short_break_minutes, long_break_minutes, sessions_until_long_break, alarm_volume, alarm_duration_seconds, alarm_file_path);
        let todo = Todo::new(save_path);
        
        // Load pomodoro session data from the todo file if enabled
        if config.todo.save_pomodoro_data {
            let sessions = todo.get_pomodoro_sessions().to_vec();
            timer.load_daily_sessions(sessions);
        }
        
        Ok(Self {
            app: App::new(),
            timer,
            summary: Summary::new(daily_goal_minutes),
            todo,
            track_list: TrackList::new(music_dir.as_deref()),
            config,
            last_key_time: Instant::now(),
            last_key_code: None,
            was_alarm_active_last_update: false,
        })
    }
    
    /// Reload configuration from file and apply changes
    fn reload_config(&mut self) -> Result<()> {
        self.config.reload()?;
        
        // Apply configuration changes to components
        self.track_list.update_music_directory(self.config.music.music_directory.as_deref());
        
        Ok(())
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_state = AppState::new()?;
    let result = run(terminal, app_state);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal, mut app_state: AppState) -> Result<()> {
    loop {
        terminal.draw(|frame| render(frame, &mut app_state))?;
        
        // Update music playback state (check for track finished, auto-advance)
        app_state.track_list.update_playback_state();
        
        // Coordinate music volume with alarm state
        let is_alarm_active = app_state.timer.update_alarm_state();
        
        if is_alarm_active && !app_state.was_alarm_active_last_update {
            // Alarm just started - lower music volume
            app_state.track_list.lower_volume_for_alarm(app_state.timer.get_alarm_volume());
        } else if !is_alarm_active && app_state.was_alarm_active_last_update {
            // Alarm just ended - restore normal music volume
            app_state.track_list.restore_volume(app_state.config.music.default_volume);
        }
        
        app_state.was_alarm_active_last_update = is_alarm_active;
        
        // Use timeout when timer is running, poll immediately when stopped
        let timeout = if matches!(app_state.timer.state, timer::TimerState::Running) {
            std::time::Duration::from_millis(100) // Update 10 times per second when running
        } else {
            std::time::Duration::from_millis(1000) // Check once per second when stopped
        };
        
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Only handle key press events, ignore key release events
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                
                // Debounce key events to prevent double-triggering, but skip debouncing for Chinese characters
                // This allows Chinese IME input to work properly while preventing accidental repeated key presses
                let now = Instant::now();
                let should_debounce = if let KeyCode::Char(c) = key.code {
                    // Never debounce Chinese characters
                    if is_chinese_character(c) {
                        false
                    } else {
                        // For non-Chinese characters, debounce identical keys
                        if let Some(last_key) = app_state.last_key_code {
                            last_key == key.code && 
                            now.duration_since(app_state.last_key_time) < std::time::Duration::from_millis(50)
                        } else {
                            false
                        }
                    }
                } else {
                    // For non-character keys, use normal debouncing
                    if let Some(last_key) = app_state.last_key_code {
                        last_key == key.code && 
                        now.duration_since(app_state.last_key_time) < std::time::Duration::from_millis(50)
                    } else {
                        false
                    }
                };
                
                if should_debounce {
                    continue;
                }
                
                app_state.last_key_time = now;
                app_state.last_key_code = Some(key.code);
                
                // Handle help popup first (global key)
            match key.code {
                KeyCode::Char('?') => {
                    app_state.app.toggle_help();
                    continue;
                }
                KeyCode::Esc => {
                    if app_state.app.show_help {
                        app_state.app.close_help();
                        continue;
                    } else if app_state.todo.is_input_mode {
                        app_state.todo.cancel_input_mode();
                        continue;
                    }
                }
                _ => {}
            }
            
            // Skip other inputs if help is shown
            if app_state.app.show_help {
                // Handle help-specific controls
                match key.code {
                    KeyCode::Char('j') | KeyCode::Down => {
                        let lines: Vec<&str> = Help::get_content().lines().collect();
                        let visible_lines = 20; // Approximate visible lines in help popup
                        app_state.app.help.scroll_down(lines.len(), visible_lines);
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        app_state.app.help.scroll_up();
                    }
                    KeyCode::Char('+') => {
                        app_state.app.help.increase_width();
                    }
                    KeyCode::Char('-') => {
                        app_state.app.help.decrease_width();
                    }
                    KeyCode::Char('=') => {
                        app_state.app.help.increase_height();
                    }
                    KeyCode::Char('_') => {
                        app_state.app.help.decrease_height();
                    }
                    _ => {}
                }
                continue;
            }
            
            // Check if we're in todo input mode
            if app_state.todo.is_input_mode {
                match key.code {
                    KeyCode::Enter => {
                        app_state.todo.submit_new_task();
                    }
                    KeyCode::Backspace => {
                        app_state.todo.remove_char_from_input();
                    }
                    KeyCode::Char(c) => {
                        app_state.todo.add_char_to_input(c);
                    }
                    _ => {}
                }
            } else {
                // Normal navigation and command mode
                match key.code {
                    KeyCode::Char('q') => {
                        // Save pomodoro session data before exiting
                        if app_state.config.todo.save_pomodoro_data {
                            let sessions = app_state.timer.get_daily_sessions().to_vec();
                            app_state.todo.save_pomodoro_sessions(sessions);
                        }
                        break Ok(());
                    }
                    
                    // h and l for cycling between panels horizontally
                    KeyCode::Char('h') => {
                        app_state.app.cycle_panels('h');
                    }
                    KeyCode::Char('l') => {
                        app_state.app.cycle_panels('l');
                    }
                    KeyCode::Char('j') => {
                        // Move down within the current panel only
                        match app_state.app.focused_quadrant {
                            Quadrant::BottomLeft => {
                                // Navigate within todo items
                                app_state.todo.move_selection_down();
                            }
                            Quadrant::BottomRight => {
                                // Navigate within track list
                                app_state.track_list.move_selection_down();
                            }
                            _ => {
                                // Other panels don't have internal navigation yet
                            }
                        }
                    }
                    KeyCode::Char('k') => {
                        // Move up within the current panel only
                        match app_state.app.focused_quadrant {
                            Quadrant::BottomLeft => {
                                // Navigate within todo items
                                app_state.todo.move_selection_up();
                            }
                            Quadrant::BottomRight => {
                                // Navigate within track list
                                app_state.track_list.move_selection_up();
                            }
                            _ => {
                                // Other panels don't have internal navigation yet
                            }
                        }
                    }
                    KeyCode::Char('a') => {
                        // Only start input mode if focused on todo quadrant
                        if app_state.app.focused_quadrant == Quadrant::BottomLeft {
                            app_state.todo.start_input_mode();
                        }
                    }
                    KeyCode::Char('d') => {
                        // Toggle done status of selected todo item
                        if app_state.app.focused_quadrant == Quadrant::BottomLeft {
                            app_state.todo.toggle_selected_task();
                        }
                    }
                    KeyCode::Char('D') => {
                        // Delete selected todo item
                        if app_state.app.focused_quadrant == Quadrant::BottomLeft {
                            app_state.todo.delete_selected_task();
                        }
                    }
                    KeyCode::Char('s') => {
                        // Select todo item for timer and add focused time
                        if app_state.app.focused_quadrant == Quadrant::BottomLeft {
                            if let Some(selected_task) = app_state.todo.get_selected_task() {
                                // Set the selected TODO item in the timer with task name
                                app_state.timer.set_selected_todo_with_task_name(
                                    Some(app_state.todo.selected_index), 
                                    Some(selected_task.task.clone())
                                );
                                
                                // Start the timer if it's not running
                                if matches!(app_state.timer.state, timer::TimerState::Stopped) {
                                    app_state.timer.toggle_start_pause();
                                }
                            }
                        }
                    }
                    KeyCode::Enter => {
                        // Play selected track when focused on track list
                        if app_state.app.focused_quadrant == Quadrant::BottomRight {
                            app_state.track_list.play_selected();
                        }
                    }
                    KeyCode::Char(' ') => {
                        // Space - Toggle start/pause timer when focused on timer, or play/pause music when focused on track list
                        match app_state.app.focused_quadrant {
                            Quadrant::TopLeft => {
                                app_state.timer.toggle_start_pause();
                            }
                            Quadrant::BottomRight => {
                                app_state.track_list.toggle_play_pause();
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char('r') => {
                        // Reset timer when focused on timer
                        if app_state.app.focused_quadrant == Quadrant::TopLeft {
                            app_state.timer.reset();
                        }
                    }
                    KeyCode::Char('S') => {
                        // Skip to next phase when focused on timer (capital S)
                        if app_state.app.focused_quadrant == Quadrant::TopLeft {
                            app_state.timer.skip_phase();
                        }
                    }
                    KeyCode::Char('z') => {
                        // Undo last action in todo
                        if app_state.app.focused_quadrant == Quadrant::BottomLeft {
                            app_state.todo.undo();
                        }
                    }
                    KeyCode::Char('n') => {
                        // Next track when focused on track list
                        if app_state.app.focused_quadrant == Quadrant::BottomRight {
                            app_state.track_list.next_track();
                        }
                    }
                    KeyCode::Char('p') => {
                        // Previous track when focused on track list
                        if app_state.app.focused_quadrant == Quadrant::BottomRight {
                            app_state.track_list.previous_track();
                        }
                    }
                    KeyCode::Char('R') => {
                        // Refresh music library when focused on track list (capital R)
                        if app_state.app.focused_quadrant == Quadrant::BottomRight {
                            app_state.track_list.refresh_library();
                        }
                    }
                    KeyCode::Char('m') => {
                        // Cycle playback mode when focused on track list
                        if app_state.app.focused_quadrant == Quadrant::BottomRight {
                            app_state.track_list.cycle_playback_mode();
                        }
                    }
                    KeyCode::PageUp => {
                        // Page up in todo list
                        if app_state.app.focused_quadrant == Quadrant::BottomLeft {
                            app_state.todo.page_up();
                        }
                    }
                    KeyCode::PageDown => {
                        // Page down in todo list
                        if app_state.app.focused_quadrant == Quadrant::BottomLeft {
                            app_state.todo.page_down();
                        }
                    }
                    KeyCode::Char('C') => {
                        // Reload configuration (capital C)
                        if let Err(e) = app_state.reload_config() {
                            // In a real app, you might want to show this error to the user
                            eprintln!("Failed to reload config: {}", e);
                        }
                    }
                    _ => {}
                }
            }
            } // Close the if let Event::Key(key) block
        } // This closes the if event::poll() block
        // Continue the loop even if no event occurred (for timer updates)
    }
}

fn render(frame: &mut Frame, app_state: &mut AppState) {
    // Fill the background with Dracula background color
    let bg_block = Block::default().style(Style::default().bg(DraculaTheme::BACKGROUND));
    frame.render_widget(bg_block, frame.area());
    
    // Check if a work phase just completed and add time to the selected TODO
    if app_state.timer.work_phase_just_completed() {
        if let Some(todo_index) = app_state.timer.get_selected_todo() {
            let work_minutes = app_state.timer.get_work_session_minutes();
            app_state.todo.add_time_to_task_by_index(todo_index, work_minutes);
            // Clear the selected todo and flag after adding time
            app_state.timer.set_selected_todo(None);
            app_state.timer.clear_work_completed_flag();
        }
    }

    // Create main vertical layout (top and bottom)
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(frame.area());

    // Create top horizontal layout (top-left and top-right)
    let top_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_layout[0]);

    // Create bottom horizontal layout (bottom-left and bottom-right)
    let bottom_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_layout[1]);

    // Render each component in its respective area
    app_state.timer.render(frame, top_layout[0], &app_state.app, &app_state.todo.items);
    app_state.summary.render(frame, top_layout[1], &app_state.app, &app_state.todo);
    app_state.todo.render(frame, bottom_layout[0], &app_state.app);
    app_state.track_list.render(frame, bottom_layout[1], &app_state.app);
    
    // Render help popup on top if shown
    if app_state.app.show_help {
        app_state.app.help.render(frame);
    }
}
