use std::{sync::Arc, thread};

use clone_macro::clone;
use const_format::concatcp;
use egui_phosphor::regular::CARET_RIGHT;
use nalgebra::Vector3;
use parking_lot::Mutex;
use tracing::info;

use crate::{
    core::{
        App,
        slice::{InteractivePreviews, SliceOperation},
        state::PreviewModel,
    },
    interface::{
        panels::Tab,
        popup::{Popup, PopupIcon},
    },
    render::camera::Camera,
};
use common::{progress::CombinedProgress, slice::SliceMode, units::Milimeter};
use slicer::{
    mesh::Mesh,
    slicer::{Slicer, SlicerModel},
};

impl App {
    pub fn slice(&mut self) {
        let meshes = (self.project.models.iter_mut())
            .filter(|x| !x.hidden)
            .map(|x| (x.supports.mesh().clone(), x.clone()))
            .collect::<Vec<_>>();

        if meshes.is_empty() {
            const NO_MODELS_ERROR: &str = concatcp!(
                "There are no models to slice. Add one by going to File ",
                CARET_RIGHT,
                " Import Model or drag and drop a model file into the workspace."
            );
            self.popup.open(Popup::simple(
                "Empty Workspace",
                PopupIcon::Error,
                NO_MODELS_ERROR,
            ));
            return;
        }

        info!("Starting slicing operation");

        let slice_config = self.project.slice_config.clone();
        let slice_height = slice_config.slice_height.get::<Milimeter>();
        let platform_size = (slice_config.platform_size.xy()).map(|x| x.get::<Milimeter>());

        let platform = slice_config.platform_resolution.cast::<f32>();
        let mm_to_px = platform.component_div(&platform_size).push(1.0);

        let device = &self.render_state.device;

        // Transform models from world-space to platform-space
        let mut out = Vec::new();
        let mut preview = Vec::new();

        let mut triangles = 0;
        let (mut min, mut max) = (Vector3::repeat(f32::MAX), Vector3::repeat(f32::MIN));

        for (supports, mut model) in meshes.into_iter() {
            // preview
            let (model_min, model_max) = model.mesh.bounds();
            min = min.zip_map(&model_min, f32::min);
            max = max.zip_map(&model_max, f32::max);
            preview.push(PreviewModel::for_model(&mut model, device));

            // slicing
            let (mut mesh, exposure) = (model.mesh, model.exposure);
            let offset = (platform / 2.0).push(-slice_height / 2.0);

            transform_mesh(&mut mesh, mm_to_px, offset);
            triangles += mesh.face_count() as u64;
            out.push(SlicerModel { mesh, exposure });

            if let Some((mut mesh, _)) = supports {
                transform_mesh(&mut mesh, mm_to_px, offset);
                triangles += mesh.face_count() as u64;
                out.push(SlicerModel { mesh, exposure });
            }
        }

        let fov = self.config.render.preview.fov;
        let camera = Camera {
            target: (max + min) / 2.0,
            distance: (max + min).magnitude() / 4.0 / (fov / 2.0).tan(),
            fov,
            ..Default::default()
        };

        let previews = InteractivePreviews {
            camera: Mutex::new(camera),
            models: preview,
        };

        let slicer = Slicer::new(slice_config, out);
        let post_process = CombinedProgress::new();
        let slice_operation =
            SliceOperation::new(slicer.progress(), post_process.clone(), Some(previews));
        self.slice_operation.replace(slice_operation);
        self.panels.focus_tab(Tab::Sliced);

        let triangles = Some(triangles);
        thread::spawn(clone!(
            [
                { self.slice_operation } as slice_operation,
                { self.project.post_processing } as post_processing
            ],
            move || {
                let slice_operation = slice_operation.as_ref().unwrap();

                match slicer.slice_config.mode {
                    SliceMode::Raster => {
                        let mut layers = slicer.slice_raster();
                        post_processing.process(&slicer.slice_config, &mut layers, post_process);
                        slice_operation.add_raster_result(slicer.slice_config, layers, triangles);
                    }
                    SliceMode::Vector => {
                        let layers = slicer.slice_vector();
                        slice_operation.add_vector_result(
                            slicer.slice_config,
                            Arc::new(layers),
                            triangles,
                        );
                    }
                }
            }
        ));
    }
}

fn transform_mesh(mesh: &mut Mesh, mm_to_px: Vector3<f32>, offset: Vector3<f32>) {
    mesh.set_scale_unchecked(mesh.scale().component_mul(&mm_to_px));
    mesh.set_position_unchecked(mesh.position().component_mul(&mm_to_px) + offset);
    mesh.update_transformation_matrix();
}
