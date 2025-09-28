use crate::help::Help;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Quadrant {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub struct App {
    pub focused_quadrant: Quadrant,
    pub show_help: bool,
    pub help: Help,
}

impl App {
    pub fn new() -> Self {
        Self {
            focused_quadrant: Quadrant::TopLeft,
            show_help: false,
            help: Help::new(),
        }
    }
    
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
    
    pub fn close_help(&mut self) {
        self.show_help = false;
    }

    pub fn navigate(&mut self, direction: char) {
        self.focused_quadrant = match (self.focused_quadrant, direction) {
            // h - move left
            (Quadrant::TopRight, 'h') => Quadrant::TopLeft,
            (Quadrant::BottomRight, 'h') => Quadrant::BottomLeft,
            // l - move right
            (Quadrant::TopLeft, 'l') => Quadrant::TopRight,
            (Quadrant::BottomLeft, 'l') => Quadrant::BottomRight,
            // k - move up
            (Quadrant::BottomLeft, 'k') => Quadrant::TopLeft,
            (Quadrant::BottomRight, 'k') => Quadrant::TopRight,
            // j - move down
            (Quadrant::TopLeft, 'j') => Quadrant::BottomLeft,
            (Quadrant::TopRight, 'j') => Quadrant::BottomRight,
            // No movement if at edge
            _ => self.focused_quadrant,
        };
    }

    /// Cycle through panels horizontally: timer → summary → todo → music → timer
    pub fn cycle_panels(&mut self, direction: char) {
        match direction {
            'l' => {
                // Move right in cycle: timer → summary → todo → music → timer
                self.focused_quadrant = match self.focused_quadrant {
                    Quadrant::TopLeft => Quadrant::BottomLeft,     // timer → todo
                    Quadrant::BottomLeft => Quadrant::TopRight, // todo → summary
                    Quadrant::TopRight => Quadrant::BottomRight,  // summary → music
                    Quadrant::BottomRight => Quadrant::TopLeft,  // music → timer
                };
            }
            'h' => {
                // Move left in cycle: timer ← summary ← todo ← music ← timer
                self.focused_quadrant = match self.focused_quadrant {
                    Quadrant::TopLeft => Quadrant::BottomRight,  // timer ← music
                    Quadrant::TopRight => Quadrant::BottomLeft,     // summary ← todo
                    Quadrant::BottomLeft => Quadrant::TopLeft,  // todo ← timer
                    Quadrant::BottomRight => Quadrant::TopRight, // music ← summary
                };
            }
            _ => {}
        }
    }
}