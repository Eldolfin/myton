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
