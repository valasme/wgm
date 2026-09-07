//! The state every command reaches through.

use crate::document::Store;
use crate::paths::DataPaths;
use crate::settings::Settings;
use crate::workspace::WorkspaceState;

pub struct AppState {
    pub paths: DataPaths,
    pub settings: Store<Settings>,
    pub workspace: Store<WorkspaceState>,
}

impl AppState {
    /// Resolve the data directory and open both documents.
    ///
    /// Two documents on one module is what proves the document module is genuinely
    /// generic. If a third were needed tomorrow it would be one more line here.
    pub fn load() -> Self {
        let paths = crate::paths::resolve();
        let ephemeral = paths.is_ephemeral();

        AppState {
            settings: Store::open(&paths.data_dir, ephemeral),
            workspace: Store::open(&paths.data_dir, ephemeral),
            paths,
        }
    }

    /// True when *anything* is preventing persistence: no writable directory, or
    /// either document written by a newer build.
    pub fn is_ephemeral(&self) -> bool {
        self.paths.is_ephemeral() || self.settings.is_ephemeral() || self.workspace.is_ephemeral()
    }
}
