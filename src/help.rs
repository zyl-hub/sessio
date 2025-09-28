use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::theme::DraculaTheme;

pub struct Help {
    pub scroll_offset: usize,
    pub width_percent: u16,
    pub height_percent: u16,
}

impl Help {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            width_percent: 85,
            height_percent: 85,
        }
    }

    pub fn get_content() -> &'static str {
        r#"🚀 PRODUCTIVITY SUITE - HELP

📋 GENERAL NAVIGATION:
  h/l     - Cycle between panels: timer→summary→todo→music→timer
  j/k     - Navigate within current panel (up/down)
  q       - Quit application
  ?       - Toggle this help (ESC to close)
  C       - Reload configuration file

⏱️  TIMER PANEL (Top-Left):
  Space   - Start/Pause timer
  r       - Reset current timer
  S       - Skip to next phase
  • Plays alarm sound when timer ends (place alarm.wav in ~/.config/sessio/)

✅ TODO PANEL (Bottom-Left):
  j/k     - Navigate within todo items  
  a       - Add new task
  d       - Toggle done status
  D       - Delete selected task
  s       - Select task for timer (starts timer)
  z       - Undo last action
  PgUp/Dn - Page up/down in todo list

📊 SUMMARY PANEL (Top-Right):
  Shows daily statistics, streaks, and progress

🎵 TRACK LIST PANEL (Bottom-Right):
  j/k     - Navigate within track list
  Space   - Play/Pause current track
  Enter   - Play selected track
  n       - Next track
  p       - Previous track
  m       - Cycle playback mode (Track List/Random/Repeat/Current Only)
  R       - Refresh music library

🍅 POMODORO TECHNIQUE:
  • 25min work sessions
  • 5min short breaks  
  • 15min long breaks (every 4th session)
  • Time automatically tracked to selected todo

⚙️  CONFIGURATION:
  • Config file: ~/.config/sessio/sessio.toml
  • Automatically created with defaults on first run
  • Reload with 'C' key without restarting
  • See sessio.toml.example for all options

📈 FEATURES:
  • Timeline tracking in markdown
  • Daily/weekly statistics  
  • Streak counting
  • Automatic time logging
  • Persistent todo storage

🔧 HELP PANEL CONTROLS:
  j/k or ↓/↑ - Scroll up/down
  +/-        - Increase/decrease width
  =/−        - Increase/decrease height
  ESC        - Close help

Press ESC to close this help"#
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self, max_lines: usize, visible_lines: usize) {
        if self.scroll_offset + visible_lines < max_lines {
            self.scroll_offset += 1;
        }
    }

    pub fn increase_width(&mut self) {
        if self.width_percent < 95 {
            self.width_percent += 5;
        }
    }

    pub fn decrease_width(&mut self) {
        if self.width_percent > 50 {
            self.width_percent -= 5;
        }
    }

    pub fn increase_height(&mut self) {
        if self.height_percent < 95 {
            self.height_percent += 5;
        }
    }

    pub fn decrease_height(&mut self) {
        if self.height_percent > 50 {
            self.height_percent -= 5;
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let help_content = Self::get_content();

        // Split content into lines for scrolling
        let lines: Vec<&str> = help_content.lines().collect();
        let total_lines = lines.len();
        
        // Calculate popup size and position
        let area = frame.area();
        let popup_area = Self::centered_rect(self.width_percent, self.height_percent, area);
        let inner_area = Block::default().borders(Borders::ALL).inner(popup_area);
        let visible_lines = inner_area.height.saturating_sub(1) as usize; // Reserve 1 line for potential scroll indicator

        // Clear the background
        frame.render_widget(Clear, popup_area);
        
        // Calculate visible content based on scroll offset
        let end_line = (self.scroll_offset + visible_lines).min(total_lines);
        let visible_content = lines[self.scroll_offset..end_line].join("\n");
        
        // Add scroll indicator if there's more content
        let scroll_indicator = if total_lines > visible_lines {
            format!("\n[Scroll: {}/{}] Use j/k to scroll, +/- for width, =/- for height", 
                    self.scroll_offset + 1, 
                    total_lines.saturating_sub(visible_lines) + 1)
        } else {
            String::new()
        };
        
        let final_content = format!("{}{}", visible_content, scroll_indicator);
        
        // Create the help popup
        let help_block = Block::default()
            .title("❓ Help & Keybindings")
            .title_style(Style::default().fg(DraculaTheme::PINK))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(DraculaTheme::PINK))
            .style(Style::default().bg(DraculaTheme::CURRENT_LINE).fg(DraculaTheme::FOREGROUND));

        let help_paragraph = Paragraph::new(final_content)
            .block(help_block)
            .style(Style::default().fg(DraculaTheme::FOREGROUND).bg(DraculaTheme::CURRENT_LINE))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        frame.render_widget(help_paragraph, popup_area);
    }

    /// Helper function to create a centered rect using up to certain percentage of the available rect
    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}