/// Terminal UI module (future: ratatui-based dashboard).
pub struct JarvisTui;

impl JarvisTui {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JarvisTui {
    fn default() -> Self {
        Self::new()
    }
}
