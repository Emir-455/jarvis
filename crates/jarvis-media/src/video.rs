/// Video editing engine (stub — wraps FFmpeg in production).
pub struct VideoEditor;

impl VideoEditor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for VideoEditor {
    fn default() -> Self {
        Self::new()
    }
}
