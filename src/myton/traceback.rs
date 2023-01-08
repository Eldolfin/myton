#[derive(Debug, Clone)]
pub struct Traceback {
    pub pos: (usize, usize),
    pub message: Option<String>,
    pub filename: Option<String>,
    pub function_name: Option<String>,
    pub code: Option<String>,
}

impl Default for Traceback {
    fn default() -> Self {
        Self {
            pos: (0, 0),
            message: None,
            filename: None,
            function_name: None,
            code: None,
        }
    }
}

impl Traceback {
    pub fn from_message(message: &str) -> Self {
        Self {
            message: Some(message.to_string()),
            ..Default::default()
        }
    }
}
