use crate::app::slice_operation::SliceResult;
use crate::post_processing::{Pass, PassProgress};
use common::annotations::{Annotation, Annotations, Cluster, ClusterView};
use common::config::SliceConfig;
use const_format::concatcp;
use egui::{Color32, Grid, RichText};
use egui_phosphor::regular::{CHECK_FAT, ISLAND};
use goo_format::LayerDecoder;
use itertools::Itertools;
use parking_lot::Mutex;
use rayon::prelude::*;
use slicer::format::FormatSliceFile;
use std::f32::consts::PI;
use std::thread::JoinHandle;
use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

use super::{AnalysisReport, PassOutput, PassRunningGuard, PassState};

mod slice_coord;
pub use slice_coord::SliceCoord;

#[derive(Default)]
pub struct IslandDetectionPass {
    radius: usize,
    state: Arc<RwLock<PassState>>,
    result: Arc<Mutex<Option<PassOutput>>>,
}

impl IslandDetectionPass {
    fn max_angle(radius: usize, cfg: &SliceConfig) -> f32 {
        let pixel_length = cfg.platform_size.x / cfg.platform_resolution.x as f32;
        let pixel_width = cfg.platform_size.y / cfg.platform_resolution.y as f32;
        let pixel_height = cfg.slice_height;
        let r = radius as f32 * pixel_length.max(pixel_width);
        (r / (r * r + pixel_height * pixel_height).sqrt()).asin() * 180.0_f32 / PI
    }
}

impl Pass for IslandDetectionPass {
    fn name(&self) -> &'static str {
        "Island Detection"
    }

    fn description(&self) -> &'static str {
        "Detects islands and missing supports in the final slices."
    }

    fn ui(&mut self, ui: &mut egui::Ui, cfg: &SliceConfig) {
        ui.label(RichText::new("Island Detection").size(32.0));
        ui.add_space(8.0);
        ui.label(RichText::new(self.description()).italics());
        ui.label("Detects islands in the sliced model.");
        Grid::new("knobs")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label("Connectivity radius");
                ui.add(
                    egui::DragValue::new(&mut self.radius)
                        .speed(0.1)
                        .suffix("px")
                        .clamp_range(1..=50),
                );
                ui.label(format!(
                    "max. angle: {:.2}°",
                    Self::max_angle(self.radius, cfg)
                ));
            });
    }

    fn run(
        &mut self,
        slice_config: &common::config::SliceConfig,
        slice_result: Arc<Mutex<Option<SliceResult>>>,
        progress: Arc<Mutex<PassProgress>>,
    ) -> JoinHandle<()> {
        let state = self.state.clone();
        let radius = self.radius as i16;
        let cfg = slice_config.clone();
        let res = slice_result.clone();
        let pass_res = self.result().clone();
        std::thread::spawn(move || {
            Self::detect_islands(state, radius, cfg, res, progress, pass_res)
        })
    }

    fn state(&self) -> Arc<RwLock<PassState>> {
        self.state.clone()
    }

    fn result(&self) -> Arc<Mutex<Option<PassOutput>>> {
        self.result.clone()
    }
}

impl IslandDetectionPass {
    fn detect_islands(
        state: Arc<RwLock<PassState>>,
        radius: i16,
        cfg: common::config::SliceConfig,
        slice_result: Arc<Mutex<Option<SliceResult>>>,
        progress: Arc<Mutex<PassProgress>>,
        result: Arc<Mutex<Option<PassOutput>>>,
    ) {
        let _guard = PassRunningGuard::new(state);
        progress.lock().progress = 0.0_f32;
        let annotations = Arc::new(Mutex::new(Annotations::default()));
        let layers = {
            slice_result
                .lock()
                .as_ref()
                .and_then(|res| match &res.file {
                    FormatSliceFile::Goo(file) => Some(file),
                    _ => None,
                })
                .map(|goo| goo.layers.clone())
        }
        .unwrap();
        let sparse_maps: Vec<_> = layers
            .par_iter()
            .enumerate()
            .map(|(idx, layer)| {
                let runs = LayerDecoder::new(layer.data.as_slice());
                let mut set: HashSet<SliceCoord> = HashSet::new();
                let mut curr = SliceCoord::new(&cfg);
                for run in runs {
                    if run.value == 0 {
                        curr = curr + run.length as usize;
                    } else {
                        for _ in 0..run.length {
                            set.insert(curr);
                            curr = curr + 1;
                        }
                    }
                }
                let mut p = progress.lock();
                p.progress += 0.3_f32 / layers.len() as f32;
                p.message = "preparing layer data...".into();
                (idx, set)
            })
            .collect();
        tracing::debug!("progress at end of phase 1: {}", progress.lock().progress);
        let pairs = sparse_maps.windows(2).collect::<Vec<_>>();
        let num_pairs = pairs.len();
        pairs
                .into_par_iter()
                .for_each(|layers| {
                    if let [(idx0, l0), (idx1, l1)] = layers {
                        for coord in l1 {
                            let is_connected = coord.neighborhood(radius)
                                .any(|c| l0.contains(&(coord.cfg, c).into()));
                            if !is_connected {
                                let island = Annotation::Island { slice_idx: *idx1 + 1, coord: [coord.x() as i64, coord.y() as i64], };
                                annotations.lock().push(island.as_error());
                                tracing::trace!("island found on layer #{idx1}: {coord} not connected to layer #{idx0} within a radius of {}", radius);
                            }
                        }
                    }
                    let mut p = progress.lock();
                    p.progress += 0.6_f32 / num_pairs as f32;
                    p.message = "detecting islands...".into();
                });
        tracing::debug!("progress at end of phase 2: {}", progress.lock().progress);
        progress.lock().message = "clustering islands...".into();
        let report: IslandDetectionReport = Arc::new(
            Arc::try_unwrap(annotations)
                .expect("could not unwrap annotations")
                .into_inner(),
        )
        .into();
        progress.lock().progress += 0.05_f32;
        tracing::debug!("progress at end of phase 3: {}", progress.lock().progress);
        *result.lock() = Some(PassOutput::Analysis(Box::new(report)));
        progress.lock().progress += 0.05_f32;
        tracing::debug!("progress at end of phase 4: {}", progress.lock().progress);
    }
}

pub struct IslandDetectionReport {
    annotations: Arc<Annotations>,
    clusters: ClusterView,
}

impl From<Arc<Annotations>> for IslandDetectionReport {
    fn from(value: Arc<Annotations>) -> Self {
        Self {
            annotations: value.clone(),
            clusters: ClusterView::new(value),
        }
    }
}

impl IslandDetectionReport {
    pub fn cluster_islands(&self) -> &ClusterView {
        &self.clusters
    }

    pub fn clusters(&self) -> impl Iterator<Item = &Cluster> {
        self.cluster_islands().clusters.iter()
    }
}

impl AnalysisReport for IslandDetectionReport {
    fn ui(&self, ui: &mut egui::Ui, slice_result: Arc<Mutex<Option<SliceResult>>>) {
        let ui_closure = |ui: &mut egui::Ui| {
            Grid::new("islands")
                .num_columns(1)
                .striped(true)
                .show(ui, |ui| {
                    Grid::new("island_layers")
                        .num_columns(1)
                        .striped(true)
                        .show(ui, |ui| {
                            self.clusters()
                                .flat_map(|c| c.slice_idx())
                                .collect::<HashSet<_>>()
                                .iter()
                                .sorted()
                                .for_each(|layer| {
                                    ui.collapsing(format!("Layer #{}", layer), |ui| {
                                        self.clusters()
                                            .filter(
                                                |c| matches!(c.slice_idx(), Some(l) if l == *layer),
                                            )
                                            .for_each(|c| {
                                                let center = c.center().unwrap();
                                                let label = ui.label(format!(
                                                    "{} cluster of {} pixels at ({}, {}).",
                                                    ISLAND,
                                                    c.len(),
                                                    center[0],
                                                    center[1]
                                                ));
                                                if label.clicked() {
                                                    if let Some(res) = &mut *slice_result.lock() {
                                                        res.center_on(&center);
                                                        res.set_layer_idx(*layer);
                                                    }
                                                };
                                            });
                                    });
                                    ui.end_row();
                                });
                        })
                });
        };
        if !self.annotations.is_empty() {
            ui.collapsing(
                RichText::new(format!(
                    "{} Found {} islands clusters.",
                    ISLAND,
                    self.clusters().count()
                ))
                .color(Color32::RED),
                ui_closure,
            );
        } else {
            ui.label(
                RichText::new(concatcp!(CHECK_FAT, " No islands found!")).color(Color32::GREEN),
            );
        }
    }

    fn annotations(&self) -> &Annotations {
        &self.annotations
    }
}
