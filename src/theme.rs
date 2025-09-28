use ratatui::style::Color;

// Dracula theme colors
pub struct DraculaTheme;

impl DraculaTheme {
    pub const BACKGROUND: Color = Color::Rgb(40, 42, 54);      // #282a36
    pub const CURRENT_LINE: Color = Color::Rgb(68, 71, 90);    // #44475a
    pub const FOREGROUND: Color = Color::Rgb(248, 248, 242);   // #f8f8f2
    pub const COMMENT: Color = Color::Rgb(98, 114, 164);       // #6272a4
    pub const CYAN: Color = Color::Rgb(139, 233, 253);         // #8be9fd
    pub const GREEN: Color = Color::Rgb(80, 250, 123);         // #50fa7b
    pub const ORANGE: Color = Color::Rgb(255, 184, 108);       // #ffb86c
    pub const PINK: Color = Color::Rgb(255, 121, 198);         // #ff79c6
    pub const PURPLE: Color = Color::Rgb(189, 147, 249);       // #bd93f9
    pub const RED: Color = Color::Rgb(255, 85, 85);            // #ff5555
    pub const YELLOW: Color = Color::Rgb(241, 250, 140);       // #f1fa8c
}