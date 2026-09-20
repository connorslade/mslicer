use std::path::PathBuf;

use nalgebra::Vector2;
use remote_print::manager::RemotePrintManager;
use tracing::{info, warn};

use crate::{
    config::Config,
    core::{history::History, slice_operation::SliceOperation},
    project::Project,
};

pub mod history;
mod slice;
pub mod slice_operation;

pub const SLICE_PREVIEW_SIZE: Vector2<f32> = Vector2::new(700.0, 400.0);

pub struct Core {
    pub config_dir: PathBuf,

    pub remote_print: RemotePrintManager,
    pub slice_operation: Option<SliceOperation>,

    pub history: History,

    pub config: Config,
    pub project: Project,
}

impl Core {
    pub fn init(config_dir: PathBuf, config: Config) -> Self {
        Self {
            config_dir,

            remote_print: RemotePrintManager::default(),
            slice_operation: None,
            history: History::default(),
            project: Project {
                slice_config: config.default_slice_config.clone(),
                ..Default::default()
            },
            config,
        }
    }

    pub fn is_slicing(&self) -> bool {
        is_slicing(&self.slice_operation)
    }

    pub fn update(&mut self) {
        self.history
            .set_max_mesh_size(self.config.ui.history_max_mesh_size);
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        if let Err(err) = self.config.save(&self.config_dir) {
            warn!("Failed to save config: {}", err);
        } else {
            info!("Successfully saved config");
        }
    }
}

pub fn is_slicing(slice_operation: &Option<SliceOperation>) -> bool {
    slice_operation
        .as_ref()
        .map(|x| !x.progress.complete())
        .unwrap_or_default()
}
