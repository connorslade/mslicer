use std::{sync::Arc, thread};

use clone_macro::clone;
use nalgebra::Vector3;
use tracing::info;

use crate::core::{Core, slice_operation::SliceOperation};
use common::{progress::CombinedProgress, slice::SliceMode, units::Milimeter};
use slicer::{
    mesh::Mesh,
    slicer::{Slicer, SlicerModel},
};

impl Core {
    /// Returns false if there were no models to slice
    pub fn slice(&mut self) -> bool {
        let meshes = (self.project.models.iter_mut())
            .filter(|x| !x.hidden)
            .map(|x| (x.supports.mesh().clone(), x.clone()))
            .collect::<Vec<_>>();

        if meshes.is_empty() {
            return false;
        }

        info!("Starting slicing operation");

        let slice_config = self.project.slice_config.clone();
        let slice_height = slice_config.slice_height.get::<Milimeter>();
        let platform_size = (slice_config.platform_size.xy()).map(|x| x.get::<Milimeter>());

        let platform = slice_config.platform_resolution.cast::<f32>();
        let mm_to_px = platform.component_div(&platform_size).push(1.0);

        // Transform models from world-space to platform-space
        let mut out = Vec::new();
        for (supports, model) in meshes.into_iter() {
            let (mut mesh, exposure) = (model.mesh, model.exposure);
            let offset = (platform / 2.0).push(-slice_height / 2.0);

            transform_mesh(&mut mesh, mm_to_px, offset);
            out.push(SlicerModel { mesh, exposure });

            if let Some((mut mesh, _)) = supports {
                transform_mesh(&mut mesh, mm_to_px, offset);
                out.push(SlicerModel { mesh, exposure });
            }
        }

        let slicer = Slicer::new(slice_config, out);
        let post_process = CombinedProgress::new();
        let slice_operation = SliceOperation::new(slicer.progress(), post_process.clone());
        self.slice_operation.replace(slice_operation);

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
                        slice_operation.add_raster_result(slicer.slice_config, layers);
                    }
                    SliceMode::Vector => {
                        let layers = slicer.slice_vector();
                        slice_operation.add_vector_result(slicer.slice_config, Arc::new(layers));
                    }
                }
            }
        ));

        true
    }
}

fn transform_mesh(mesh: &mut Mesh, mm_to_px: Vector3<f32>, offset: Vector3<f32>) {
    mesh.set_scale_unchecked(mesh.scale().component_mul(&mm_to_px));
    mesh.set_position_unchecked(mesh.position().component_mul(&mm_to_px) + offset);
    mesh.update_transformation_matrix();
}
