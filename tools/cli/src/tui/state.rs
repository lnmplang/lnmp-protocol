pub enum Screen {
    Convert,
    Encode,
    Decode,
    Validate,
}

pub struct AppState {
    current_screen: Screen,
    pub source_file: String,
    pub target_file: String,
    pub mapping_file: Option<String>,
    pub status_message: String,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_screen: Screen::Convert,
            source_file: String::new(),
            target_file: String::new(),
            mapping_file: None,
            status_message: "Ready".to_string(),
        }
    }

    pub fn set_screen(&mut self, screen: Screen) {
        self.current_screen = screen;
    }

    pub fn current_screen(&self) -> &Screen {
        &self.current_screen
    }

    pub fn set_status(&mut self, message: String) {
        self.status_message = message;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
