use std::{borrow::Cow, path::PathBuf};

use chrono::{DateTime, Duration, Utc};
use common::units::Milimeters;
use egui::Theme;
use egui_dock::Tree;
use serde::{Deserialize, Serialize};

use crate::interface::panels::Tab;

pub const B_PER_MIB: usize = 0x100000;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    // Panels
    pub panels: Option<Tree<Tab>>,
    pub about: bool,
    pub tasks: bool,

    // Preferences
    pub theme: Theme,
    pub history_max_mesh_size: u32, // in MiB
    pub quick_layout_spacing: Milimeters,
    pub hover_overlay: bool,
    pub hover_overlay_detail: bool,

    // Update Check
    pub update_check: UpdateCheckFrequency,
    pub last_update_check: Option<DateTime<Utc>>,
    pub ignore_update: Option<String>,

    // Tracked
    pub recents: Recents,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Recents {
    pub max: u8,

    pub pinned: Vec<RecentFile>,
    pub projects: Vec<RecentFile>,
    pub other: Vec<RecentFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentFile {
    pub path: PathBuf,
    pub date: DateTime<Utc>,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateCheckFrequency {
    Never,
    EveryLaunch,
    Daily,
    Weekly,
}

impl Recents {
    pub fn is_empty(&self) -> bool {
        self.pinned.is_empty() && self.projects.is_empty() && self.other.is_empty()
    }

    pub fn remove(&mut self, path: &PathBuf) {
        self.projects.retain(|r| &r.path != path);
        self.other.retain(|r| &r.path != path);
        self.pinned.retain(|r| &r.path != path);
    }

    pub fn remove_broken(&mut self) {
        self.projects.retain(|r| !r.path.exists());
        self.other.retain(|r| !r.path.exists());
        self.pinned.retain(|r| !r.path.exists());
    }

    pub fn toggle_pin(&mut self, path: PathBuf) {
        let pinned = self.pinned.iter().any(|r| r.path == path);
        self.remove(&path);

        if pinned {
            self.add_recent(path);
        } else {
            self.pinned.push(RecentFile::new(path));
        }
    }

    pub fn add_recent(&mut self, path: PathBuf) {
        if self.pinned.iter().any(|r| r.path == path) {
            return;
        }

        let is_project_file = path
            .extension()
            .map(|ext| ext == "mslicer")
            .unwrap_or_default();
        let list = &mut [&mut self.other, &mut self.projects][is_project_file as usize];

        list.retain(|r| r.path != path);
        while list.len() >= self.max as usize {
            list.pop();
        }

        list.insert(0, RecentFile::new(path));
    }
}

impl RecentFile {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            date: Utc::now(),
        }
    }

    pub fn name(&self) -> Cow<'_, str> {
        self.path.file_name().unwrap().to_string_lossy()
    }
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

    pub fn enabled(&self) -> bool {
        !matches!(self, UpdateCheckFrequency::Never)
    }

    pub fn toggle(&mut self) {
        *self = match self {
            UpdateCheckFrequency::Never => UpdateCheckFrequency::EveryLaunch,
            UpdateCheckFrequency::EveryLaunch
            | UpdateCheckFrequency::Daily
            | UpdateCheckFrequency::Weekly => UpdateCheckFrequency::Never,
        };
    }
}

impl Default for Recents {
    fn default() -> Self {
        Self {
            max: 5,

            pinned: Default::default(),
            projects: Default::default(),
            other: Default::default(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            panels: None,
            about: true,
            tasks: true,

            theme: Theme::Dark,
            history_max_mesh_size: 20,
            quick_layout_spacing: Milimeters::new(2.0),
            hover_overlay: true,
            hover_overlay_detail: false,

            update_check: UpdateCheckFrequency::EveryLaunch,
            last_update_check: None,
            ignore_update: None,

            recents: Default::default(),
        }
    }
}
