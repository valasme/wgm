//! Workspace State — the half of wgm's persisted state that never leaves the machine.

pub mod commands;
pub mod defaults;
pub mod geometry;
pub mod migrations;
pub mod schema;
pub mod window;

use serde_json::{Map, Value};

use crate::document::VersionedDocument;

pub use schema::WorkspaceState;

impl VersionedDocument for WorkspaceState {
    const STEM: &'static str = "workspace";
    const CURRENT_VERSION: u32 = schema::CURRENT_VERSION;
    /// **Never.** Window geometry, Sidebar width and onboarding progress cannot leak
    /// into an export or be imported from someone else's machine, and this constant
    /// is what makes that structural rather than a rule someone has to remember.
    const EXPORTABLE: bool = false;

    fn defaults() -> Self {
        defaults::defaults()
    }

    fn migrate_step(value: &mut Map<String, Value>, from: u32) -> Result<(), String> {
        migrations::step(value, from)
    }

    fn clamp(value: &mut Map<String, Value>) -> Vec<String> {
        defaults::clamp(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;

    #[test]
    fn the_two_documents_are_different_files() {
        assert_ne!(Settings::STEM, WorkspaceState::STEM);
    }

    /// A compile-time assertion rather than a runtime one, because this is the
    /// property that keeps window geometry and onboarding progress out of an export.
    /// Flipping either constant should fail the build, not a test run.
    const _: () = {
        assert!(!WorkspaceState::EXPORTABLE);
        assert!(Settings::EXPORTABLE);
    };
}
