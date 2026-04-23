mod merge;
mod settings;

pub use merge::merge_settings;
pub use settings::{load_project_settings, load_user_settings, set_config_value};

use cc_schema::Settings;

/// The effective (merged) configuration from all sources.
#[derive(Debug)]
pub struct EffectiveConfig {
    pub user: Settings,
    pub project: Settings,
    pub merged: Settings,
}
