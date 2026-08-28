use chrono::{DateTime, Duration, Utc};
use egui::Theme;
use egui_dock::Tree;
use serde::{Deserialize, Serialize};

use crate::windows::Tab;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub theme: Theme,
    pub panels: Option<Tree<Tab>>,
    pub about: bool,
    pub tasks: bool,

    pub update_check: UpdateCheckFrequency,
    pub last_update_check: Option<DateTime<Utc>>,
    pub ignore_update: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateCheckFrequency {
    Never,
    EveryLaunch,
    Daily,
    Weekly,
}

// todo: dropdown enum macro?
impl UpdateCheckFrequency {
    pub const ALL: [Self; 4] = [Self::Never, Self::EveryLaunch, Self::Daily, Self::Weekly];

    pub fn name(&self) -> &'static str {
        match self {
            UpdateCheckFrequency::Never => "Never",
            UpdateCheckFrequency::EveryLaunch => "Every Launch",
            UpdateCheckFrequency::Daily => "Daily",
            UpdateCheckFrequency::Weekly => "Weekly",
        }
    }

    pub fn as_duration(&self) -> Option<Duration> {
        Some(match self {
            UpdateCheckFrequency::Never => return None,
            UpdateCheckFrequency::EveryLaunch => Duration::zero(),
            UpdateCheckFrequency::Daily => Duration::days(1),
            UpdateCheckFrequency::Weekly => Duration::days(7),
        })
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            panels: None,
            about: true,
            tasks: true,
            update_check: UpdateCheckFrequency::EveryLaunch,
            last_update_check: None,
            ignore_update: None,
        }
    }
}
