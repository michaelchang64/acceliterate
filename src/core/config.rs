/// Reader configuration — controls WPM and display settings.
#[derive(Debug, Clone)]
pub struct ReaderConfig {
    /// Words per minute target.
    pub wpm: u32,
}

impl Default for ReaderConfig {
    fn default() -> Self {
        Self { wpm: 300 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_wpm_is_300() {
        let config = ReaderConfig::default();
        assert_eq!(config.wpm, 300);
    }
}
