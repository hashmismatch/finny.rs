use std::path::PathBuf;

/// The environment variable that overrides [`InspectorConfig::assets_dir`].
pub const ASSETS_DIR_ENV: &str = "FINNY_INSPECT_ASSETS_DIR";

/// The configuration of the [`Inspector`](crate::Inspector).
#[derive(Debug, Clone)]
pub struct InspectorConfig {
    /// How many snapshots are kept for each FSM instance. A UI that connects later can trace
    /// the history back this far. Default: 100.
    pub history_len: usize,
    /// Serve the frontend's files from this directory instead of the ones embedded in the
    /// binary, for working on the frontend without recompiling. Defaults to the value of the
    /// `FINNY_INSPECT_ASSETS_DIR` environment variable.
    pub assets_dir: Option<PathBuf>
}

impl Default for InspectorConfig {
    fn default() -> Self {
        Self {
            history_len: 100,
            assets_dir: std::env::var_os(ASSETS_DIR_ENV).map(PathBuf::from)
        }
    }
}

impl InspectorConfig {
    pub fn history_len(mut self, history_len: usize) -> Self {
        self.history_len = history_len;
        self
    }

    pub fn assets_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.assets_dir = Some(dir.into());
        self
    }
}
