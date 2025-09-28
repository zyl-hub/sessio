use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, Quadrant};
use crate::theme::DraculaTheme;
use crate::todo::Todo;

pub struct Summary {
    pub daily_goal_minutes: u32, // Daily focus time goal in minutes
}

impl Summary {
    pub fn new(daily_goal_minutes: u32) -> Self {
        Self {
            daily_goal_minutes: daily_goal_minutes, // Default to 2 hours per day
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, app: &App, todo: &Todo) {
        let is_focused = app.focused_quadrant == Quadrant::TopRight;
        
        // Get statistics
        let today_minutes = todo.get_today_minutes();
        let yesterday_minutes = todo.get_yesterday_minutes();
        let streak_days = todo.get_streak_days();
        let completed_tasks = todo.get_completed_tasks_count();
        
        // Calculate progress towards daily goal
        let goal_progress = if self.daily_goal_minutes > 0 {
            (today_minutes as f32 / self.daily_goal_minutes as f32 * 100.0).min(100.0) as u32
        } else {
            0
        };
        
        // Format time
        let today_hours = today_minutes / 60;
        let today_mins = today_minutes % 60;
        let yesterday_hours = yesterday_minutes / 60;
        let yesterday_mins = yesterday_minutes % 60;
        let goal_hours = self.daily_goal_minutes / 60;
        let goal_mins = self.daily_goal_minutes % 60;
        
        let content = format!(
            "\n🎯 Today's Progress:\n• Completed minutes: {} ({}h {}m)\n• Daily goal: {}h {}m\n• Progress: {}%\n\n📈 Statistics:\n• Yesterday: {}h {}m\n• Streak: {} days\n• Tasks completed: {}",
            today_minutes, today_hours, today_mins,
            goal_hours, goal_mins,
            goal_progress,
            yesterday_hours, yesterday_mins,
            streak_days,
            completed_tasks
        );
        
        let summary_widget = if is_focused {
            Paragraph::new(content)
                .style(Style::default().fg(DraculaTheme::FOREGROUND).bg(DraculaTheme::BACKGROUND))
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("📊 Summary")
                    .title_style(Style::default().fg(DraculaTheme::CYAN))
                    .border_style(Style::default().fg(DraculaTheme::PINK))
                    .style(Style::default().bg(DraculaTheme::BACKGROUND)))
        } else {
            Paragraph::new(content)
                .style(Style::default().fg(DraculaTheme::FOREGROUND).bg(DraculaTheme::BACKGROUND))
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("📊 Summary")
                    .title_style(Style::default().fg(DraculaTheme::CYAN))
                    .border_style(Style::default().fg(DraculaTheme::COMMENT))
                    .style(Style::default().bg(DraculaTheme::BACKGROUND)))
        };

        frame.render_widget(summary_widget, area);
    }

    // Add summary functionality methods here
    pub fn update_stats(&mut self) {
        // Update statistics logic
    }

    pub fn get_daily_summary(&self) -> String {
        // Return daily summary string
        String::from("Daily summary placeholder")
    }
}