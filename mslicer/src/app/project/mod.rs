use std::{
    iter,
    path::{Path, PathBuf},
};

use itertools::Itertools;
use tracing::error;

use crate::{
    app::{App, project::model::Model},
    ui::popup::{Popup, PopupIcon},
};
use common::config::SliceConfig;
use slicer::{
    format::FormatSliceFile,
    post_process::{anti_alias::AntiAlias, elephant_foot_fixer::ElephantFootFixer},
};

pub mod model;
mod storage;

#[derive(Default)]
pub struct Project {
    pub slice_config: SliceConfig,
    pub post_processing: PostProcessing,
    pub models: Vec<Model>,
}

#[derive(Default, Clone)]
pub struct PostProcessing {
    pub anti_alias: AntiAlias,
    pub elephant_foot_fixer: ElephantFootFixer,
}

impl App {
    fn add_recent_project(&mut self, path: PathBuf) {
        self.config.recent_projects = iter::once(path)
            .chain(self.config.recent_projects.iter().cloned())
            .unique()
            .take(5)
            .collect()
    }

    pub fn save_project(&mut self, path: &Path) {
        if let Err(error) = storage::save_project(self, path) {
            error!("Error saving project: {:?}", error);
            self.popup.open(Popup::simple(
                "Error Saving Project",
                PopupIcon::Error,
                error.to_string(),
            ));
        }
    }

    pub fn load_project(&mut self, path: &Path) {
        if let Err(error) = storage::load_project(self, path) {
            error!("Error loading project: {:?}", error);
            self.popup.open(Popup::simple(
                "Error Loading Project",
                PopupIcon::Error,
                error.to_string(),
            ));
        }
    }
}

impl PostProcessing {
    pub fn process(&self, file: &mut FormatSliceFile) {
        self.elephant_foot_fixer.post_slice(file);
        self.anti_alias.post_slice(file);
    }
}
