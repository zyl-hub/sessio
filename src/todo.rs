use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::fs;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Local, NaiveDate};

use crate::app::{App, Quadrant};
use crate::theme::DraculaTheme;

#[derive(Debug, Clone)]
pub struct TodoItem {
    pub task: String,
    pub done: bool,
    pub focused_time: u32, // in minutes
    pub timeline: Vec<WorkSession>, // Track when work was done
}

#[derive(Debug, Clone)]
pub struct WorkSession {
    pub date: NaiveDate,
    pub minutes: u32,
    pub timestamp: DateTime<Local>,
}

impl TodoItem {
    pub fn new(task: String) -> Self {
        Self {
            task,
            done: false,
            focused_time: 0,
            timeline: Vec::new(),
        }
    }
}

pub struct Todo {
    pub items: Vec<TodoItem>,
    pub is_input_mode: bool,
    pub current_input: String,
    pub file_path: String,
    pub selected_index: usize,
    pub undo_stack: Vec<Vec<TodoItem>>,
    pub scroll_offset: usize,
    pub last_visible_height: usize, // Store the last calculated visible height
}

impl Todo {
    pub fn new(save_path: Option<String>) -> Self {
        let mut todo = Self {
            items: Vec::new(),
            is_input_mode: false,
            current_input: String::new(),
            file_path: save_path.unwrap_or_else(|| "todos.md".into()),
            selected_index: 0,
            undo_stack: Vec::new(),
            scroll_offset: 0,
            last_visible_height: 8, // Default fallback value
        };
        
        // Load existing todos or create default ones
        if !todo.load_from_file() {
            // Create default items if file doesn't exist
            todo.items = vec![
                TodoItem::new("Add task management".to_string()),
                TodoItem::new("Implement priorities".to_string()),
                TodoItem::new("Set deadlines".to_string()),
            ];
            todo.save_to_file();
        }
        
        todo
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, app: &App) {
        let is_focused = app.focused_quadrant == Quadrant::BottomLeft;
        
        // Calculate available width for task text (accounting for icons, selection indicator, and padding)
        let available_width = area.width.saturating_sub(12) as usize; // Reserve space for borders, icons, etc.
        let max_task_width = available_width.saturating_sub(20); // Reserve space for time display
        
        // Calculate visible items based on available height
        let header_lines = if self.is_input_mode { 4 } else { 3 }; // Title + empty line + stats
        let footer_lines = if self.is_input_mode { 4 } else { 4 }; // Stats + help text
        let available_height = area.height.saturating_sub(header_lines + footer_lines + 2) as usize; // 2 for borders
        let visible_height = available_height.max(1); // Ensure at least 1 line is visible
        
        // Store the actual calculated visible height for use in navigation methods
        self.last_visible_height = visible_height;
        
        let visible_items: Vec<String> = if !self.items.is_empty() {
            let end_index = (self.scroll_offset + visible_height).min(self.items.len());
            self.items[self.scroll_offset..end_index]
                .iter()
                .enumerate()
                .map(|(relative_i, item)| {
                    let actual_index = self.scroll_offset + relative_i;
                    let status = if item.done { "✅" } else { "⭕" };
                    
                    // Truncate task text if too long
                    let truncated_task = if item.task.len() > max_task_width {
                        format!("{}...", &item.task[..max_task_width.saturating_sub(3)])
                    } else {
                        item.task.clone()
                    };
                    
                    let time_str = if item.focused_time > 0 {
                        format!(" ({}min)", item.focused_time)
                    } else {
                        String::new()
                    };
                    
                    let selection_indicator = if actual_index == self.selected_index && is_focused && !self.is_input_mode {
                        "►" 
                    } else { 
                        " " 
                    };
                    
                    format!("{} {} {}{}", selection_indicator, status, truncated_task, time_str)
                })
                .collect()
        } else {
            vec!["No tasks yet. Press 'a' to add one.".to_string()]
        };

        let task_list = visible_items.join("\n");

        // Show scroll indicators
        let scroll_info = if self.items.len() > visible_height {
            let showing_start = self.scroll_offset + 1;
            let showing_end = (self.scroll_offset + visible_height).min(self.items.len());
            format!(" | Showing {}-{}/{}", showing_start, showing_end, self.items.len())
        } else {
            String::new()
        };

        let content = if self.is_input_mode {
            format!("TODO - Adding New Task\n\n{}\n\n📝 {} items{}{}\n\nNew task: {}_", 
                    task_list, self.items.len(), 
                    if self.items.is_empty() { "" } else { &format!(" | Done: {}", self.items.iter().filter(|i| i.done).count()) },
                    scroll_info,
                    self.current_input)
        } else {
            let done_count = self.items.iter().filter(|i| i.done).count();
            let total_time: u32 = self.items.iter().map(|i| i.focused_time).sum();
            let selected_info = if !self.items.is_empty() {
                let selected_task = self.items.get(self.selected_index)
                    .map(|item| {
                        if item.task.len() > 30 {
                            format!("{}...", &item.task[..27])
                        } else {
                            item.task.clone()
                        }
                    })
                    .unwrap_or("None".to_string());
                format!("\n\nSelected: {}", selected_task)
            } else {
                format!("\n\nz=undo")
            };
            format!("\n{}\n\n📝 {} items | Done: {} | Total time: {}min{}{}", 
                    task_list, self.items.len(), done_count, total_time, scroll_info, selected_info)
        };

        let title = if self.is_input_mode {
            "✅ TODO - INPUT MODE"
        } else {
            "✅ TODO"
        };

        let todo_widget = if is_focused {
            Paragraph::new(content)
                .style(Style::default().fg(DraculaTheme::FOREGROUND).bg(DraculaTheme::BACKGROUND))
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .title_style(Style::default().fg(DraculaTheme::GREEN))
                    .border_style(Style::default().fg(DraculaTheme::PINK))
                    .style(Style::default().bg(DraculaTheme::BACKGROUND)))
        } else {
            Paragraph::new(content)
                .style(Style::default().fg(DraculaTheme::FOREGROUND).bg(DraculaTheme::BACKGROUND))
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .title_style(Style::default().fg(DraculaTheme::GREEN))
                    .border_style(Style::default().fg(DraculaTheme::COMMENT))
                    .style(Style::default().bg(DraculaTheme::BACKGROUND)))
        };

        frame.render_widget(todo_widget, area);
    }

    // File I/O methods
    pub fn save_to_file(&self) {
        let mut content = String::from("# TODO List\n\n");
        
        for item in &self.items {
            let checkbox = if item.done { "- [x]" } else { "- [ ]" };
            let time_info = if item.focused_time > 0 {
                format!(" | Focused time: {} minutes", item.focused_time)
            } else {
                String::new()
            };
            content.push_str(&format!("{} {}{}\n", checkbox, item.task, time_info));
            
            // Add timeline information if there are work sessions
            if !item.timeline.is_empty() {
                content.push_str("  Timeline:\n");
                for session in &item.timeline {
                    content.push_str(&format!(
                        "    - {}: {} minutes at {}\n",
                        session.date.format("%Y-%m-%d"),
                        session.minutes,
                        session.timestamp.format("%H:%M")
                    ));
                }
            }
        }
        
        // Expand ~ to home directory and create parent directories if needed
        let expanded_path = if self.file_path.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                home.join(&self.file_path[2..])
            } else {
                Path::new(&self.file_path).to_path_buf()
            }
        } else {
            Path::new(&self.file_path).to_path_buf()
        };
        
        // Create parent directories if they don't exist
        if let Some(parent) = expanded_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Failed to create directories for todos: {}", e);
                return;
            }
        }
        
        if let Err(e) = fs::write(&expanded_path, content) {
            eprintln!("Failed to save todos: {}", e);
        }
    }

    pub fn load_from_file(&mut self) -> bool {
        // Expand ~ to home directory
        let expanded_path = if self.file_path.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                home.join(&self.file_path[2..])
            } else {
                Path::new(&self.file_path).to_path_buf()
            }
        } else {
            Path::new(&self.file_path).to_path_buf()
        };
        
        if !expanded_path.exists() {
            return false;
        }
        
        match fs::read_to_string(&expanded_path) {
            Ok(content) => {
                self.items.clear();
                for line in content.lines() {
                    // Support new markdown checkbox format
                    if line.starts_with("- [x] ") || line.starts_with("- [ ] ") {
                        let done = line.starts_with("- [x]");
                        let rest = &line[6..]; // Remove "- [x] " or "- [ ] "
                        
                        if let Some(time_pos) = rest.find(" | Focused time: ") {
                            let task = rest[..time_pos].to_string();
                            let time_str = &rest[time_pos + 16..]; // Skip " | Focused time: "
                            let focused_time = time_str.split_whitespace().next()
                                .and_then(|s| s.parse::<u32>().ok())
                                .unwrap_or(0);
                            
                            self.items.push(TodoItem {
                                task,
                                done,
                                focused_time,
                                timeline: Vec::new(),
                            });
                        } else {
                            self.items.push(TodoItem {
                                task: rest.to_string(),
                                done,
                                focused_time: 0,
                                timeline: Vec::new(),
                            });
                        }
                    }
                    // Support old emoji format for backward compatibility
                    else if line.starts_with("✅ ") || line.starts_with("⭕ ") {
                        let done = line.starts_with("✅");
                        let rest = &line[4..]; // Remove status emoji and space
                        
                        if let Some(time_pos) = rest.find(" | Focused time: ") {
                            let task = rest[..time_pos].to_string();
                            let time_str = &rest[time_pos + 16..]; // Skip " | Focused time: "
                            let focused_time = time_str.split_whitespace().next()
                                .and_then(|s| s.parse::<u32>().ok())
                                .unwrap_or(0);
                            
                            self.items.push(TodoItem {
                                task,
                                done,
                                focused_time,
                                timeline: Vec::new(),
                            });
                        } else {
                            self.items.push(TodoItem {
                                task: rest.to_string(),
                                done,
                                focused_time: 0,
                                timeline: Vec::new(),
                            });
                        }
                    }
                }
                true
            }
            Err(_) => false,
        }
    }

    // Todo functionality methods
    pub fn add_task(&mut self, task: String) {
        if !task.trim().is_empty() {
            self.items.push(TodoItem::new(task));
            self.save_to_file();
        }
    }

    pub fn remove_task(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);
            self.save_to_file();
        }
    }

    pub fn toggle_task(&mut self, index: usize) {
        if index < self.items.len() {
            self.items[index].done = !self.items[index].done;
            self.save_to_file();
        }
    }

    // Undo functionality
    fn save_state_for_undo(&mut self) {
        // Keep only the last 10 states to prevent unlimited memory usage
        if self.undo_stack.len() >= 10 {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(self.items.clone());
    }

    pub fn undo(&mut self) -> bool {
        if let Some(previous_state) = self.undo_stack.pop() {
            self.items = previous_state;
            // Adjust selection index if it's out of bounds
            if self.selected_index >= self.items.len() && !self.items.is_empty() {
                self.selected_index = self.items.len() - 1;
            } else if self.items.is_empty() {
                self.selected_index = 0;
            }
            
            // Adjust scroll offset to keep selection visible
            if self.selected_index < self.scroll_offset {
                self.scroll_offset = self.selected_index;
            }
            let visible_height = self.calculate_visible_height();
            if self.selected_index >= self.scroll_offset + visible_height {
                self.scroll_offset = self.selected_index.saturating_sub(visible_height - 1);
            }
            
            self.save_to_file();
            true
        } else {
            false
        }
    }
    // Helper method to get the current visible height
    fn calculate_visible_height(&self) -> usize {
        // Use the last calculated visible height from render, with a fallback
        self.last_visible_height
    }

    pub fn move_selection_up(&mut self) {
        if !self.items.is_empty() && self.selected_index > 0 {
            self.selected_index -= 1;
            // Auto-scroll if selection goes above visible area
            if self.selected_index < self.scroll_offset {
                self.scroll_offset = self.selected_index;
            }
        }
    }

    pub fn move_selection_down(&mut self) {
        if !self.items.is_empty() && self.selected_index < self.items.len() - 1 {
            self.selected_index += 1;
            // Use dynamic visible height calculation
            let visible_height = self.calculate_visible_height();
            
            // Auto-scroll if selection goes below visible area  
            if self.selected_index >= self.scroll_offset + visible_height {
                self.scroll_offset = self.selected_index - visible_height + 1;
            }
        }
    }

    // New scrolling methods
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        let visible_height = self.calculate_visible_height();
        if self.scroll_offset + visible_height < self.items.len() {
            self.scroll_offset += 1;
        }
    }

    pub fn page_up(&mut self) {
        let page_size = 5; // Scroll by 5 items at a time
        self.scroll_offset = self.scroll_offset.saturating_sub(page_size);
    }

    pub fn page_down(&mut self) {
        let page_size = 5; // Scroll by 5 items at a time
        let visible_height = self.calculate_visible_height();
        let max_scroll = self.items.len().saturating_sub(visible_height);
        self.scroll_offset = (self.scroll_offset + page_size).min(max_scroll);
    }

    // Action methods that will be called from main.rs
    pub fn toggle_selected_task(&mut self) {
        if self.selected_index < self.items.len() {
            self.save_state_for_undo();
            self.items[self.selected_index].done = !self.items[self.selected_index].done;
            self.save_to_file();
        }
    }

    pub fn delete_selected_task(&mut self) {
        if self.selected_index < self.items.len() {
            self.save_state_for_undo();
            self.items.remove(self.selected_index);
            // Adjust selection index if needed
            if self.selected_index >= self.items.len() && !self.items.is_empty() {
                self.selected_index = self.items.len() - 1;
            } else if self.items.is_empty() {
                self.selected_index = 0;
            }
            
            // Adjust scroll offset if needed
            if self.scroll_offset > 0 && self.selected_index < self.scroll_offset {
                self.scroll_offset = self.selected_index;
            }
            
            self.save_to_file();
        }
    }

    pub fn get_selected_task(&self) -> Option<&TodoItem> {
        self.items.get(self.selected_index)
    }

    pub fn add_time_to_selected(&mut self, minutes: u32) {
        if self.selected_index < self.items.len() {
            self.save_state_for_undo();
            self.items[self.selected_index].focused_time += minutes;
            self.save_to_file();
        }
    }
    
    pub fn add_time_to_task_by_index(&mut self, index: usize, minutes: u32) {
        if index < self.items.len() {
            self.save_state_for_undo();
            self.items[index].focused_time += minutes;
            
            // Add timeline entry
            let today = chrono::Local::now().date_naive();
            let now = chrono::Local::now();
            
            // Check if there's already an entry for today, if so, update it
            if let Some(session) = self.items[index].timeline.iter_mut()
                .find(|s| s.date == today) {
                session.minutes += minutes;
                session.timestamp = now; // Update to latest work time
            } else {
                // Create new session for today
                self.items[index].timeline.push(WorkSession {
                    date: today,
                    minutes,
                    timestamp: now,
                });
            }
            
            self.save_to_file();
        }
    }
    
    // Statistics methods for summary panel
    pub fn get_today_minutes(&self) -> u32 {
        let today = chrono::Local::now().date_naive();
        self.items.iter()
            .flat_map(|item| &item.timeline)
            .filter(|session| session.date == today)
            .map(|session| session.minutes)
            .sum()
    }
    
    pub fn get_yesterday_minutes(&self) -> u32 {
        let yesterday = chrono::Local::now().date_naive() - chrono::Duration::days(1);
        self.items.iter()
            .flat_map(|item| &item.timeline)
            .filter(|session| session.date == yesterday)
            .map(|session| session.minutes)
            .sum()
    }
    
    pub fn get_streak_days(&self) -> u32 {
        let today = chrono::Local::now().date_naive();
        let dates_with_work: std::collections::HashSet<chrono::NaiveDate> = 
            self.items.iter()
                .flat_map(|item| &item.timeline)
                .map(|session| session.date)
                .collect();
        
        let mut streak = 0;
        let mut current_date = today;
        
        loop {
            if dates_with_work.contains(&current_date) {
                streak += 1;
                current_date = current_date - chrono::Duration::days(1);
            } else {
                break;
            }
        }
        
        streak
    }
    
    pub fn get_completed_tasks_count(&self) -> usize {
        self.items.iter().filter(|item| item.done).count()
    }

    pub fn start_input_mode(&mut self) {
        self.is_input_mode = true;
        self.current_input.clear();
    }

    pub fn cancel_input_mode(&mut self) {
        self.is_input_mode = false;
        self.current_input.clear();
    }

    pub fn submit_new_task(&mut self) {
        if !self.current_input.trim().is_empty() {
            self.save_state_for_undo();
            self.items.push(TodoItem::new(self.current_input.clone()));
            self.save_to_file();
        }
        self.is_input_mode = false;
        self.current_input.clear();
    }

    pub fn add_char_to_input(&mut self, c: char) {
        if self.is_input_mode {
            self.current_input.push(c);
        }
    }

    pub fn remove_char_from_input(&mut self) {
        if self.is_input_mode {
            self.current_input.pop();
        }
    }
}