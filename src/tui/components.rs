//! TUI reusable components

// Note: Imports will be added as components are implemented

pub struct TodoList {
    pub items: Vec<String>,
    pub selected: Option<usize>,
}

impl TodoList {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            items: vec![
                "Sample Todo 1".to_string(),
                "Sample Todo 2".to_string(),
                "Sample Todo 3".to_string(),
            ],
            selected: None,
        }
    }

    pub fn next(&mut self) {
        let i = match self.selected {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selected = Some(i);
    }

    pub fn previous(&mut self) {
        let i = match self.selected {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selected = Some(i);
    }
}

impl Default for TodoList {
    fn default() -> Self {
        Self::new()
    }
}