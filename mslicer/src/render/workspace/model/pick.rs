use std::{
    iter,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use clone_macro::clone;
use itertools::Itertools;
use nalgebra::Vector2;
use slicer::geometry::bounds::Bounds2D;
use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, COPY_BYTES_PER_ROW_ALIGNMENT, CommandEncoder, Device,
    Extent3d, MapMode, Origin3d, PollType, TexelCopyBufferInfo, TexelCopyBufferLayout,
    TexelCopyTextureInfo, TextureAspect,
};

use crate::{
    core::{
        App,
        project::model::ModelId,
        state::{GeometryHit, Tool},
    },
    render::{
        Gcx,
        workspace::{WorkspaceRenderResources, model::MultiStage},
    },
    util::ext::FloatExt,
};

const MODEL_ID_MASK: u32 = !(1 << 31);

pub struct ModelPicker {
    // cursor
    staging: Buffer,
    state: StateMachine,

    // area selection
    areas: Vec<(Buffer, Extent3d, Arc<AtomicBool>)>,
}

enum StateMachine {
    None,
    Copied,
    Mapping(Arc<AtomicBool>),
}

impl ModelPicker {
    pub fn new(device: &Device) -> Self {
        let staging = device.create_buffer(&BufferDescriptor {
            label: Some("Pick Staging"),
            size: size_of::<[u32; 2]>() as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Self {
            staging,
            state: StateMachine::None,

            areas: Vec::new(),
        }
    }

    pub fn update(
        &mut self,
        app: &mut App,
        gcx: &Gcx,
        encoder: &mut CommandEncoder,
        multi: &MultiStage,
    ) {
        if matches!(self.state, StateMachine::Mapping(..)) || !self.areas.is_empty() {
            let _ = gcx.device.poll(PollType::Poll);
        }

        self.areas.retain(|(buffer, extent, ready)| {
            let ready = ready.load(Ordering::Relaxed);
            ready.then(|| download_area(app, buffer, extent));

            !ready
        });

        match &self.state {
            StateMachine::None => self.copy(encoder, multi, app),
            StateMachine::Copied => {
                let complete = Arc::new(AtomicBool::new(false));
                (self.staging.slice(..)).map_async(
                    MapMode::Read,
                    clone!([complete], move |_| complete.store(true, Ordering::Relaxed)),
                );
                self.state = StateMachine::Mapping(complete);
            }
            StateMachine::Mapping(complete) => {
                complete.load(Ordering::Relaxed).then(|| self.download(app));
            }
        }
    }

    fn copy(&mut self, encoder: &mut CommandEncoder, multi: &MultiStage, app: &mut App) {
        let texture = multi.model_target.texture();
        let size = texture.size();

        let uv = app.state.workspace.uv.map(|x| x.saturate());
        let Some(pos) = uv_to_origin(size, uv) else {
            app.state.hovered_geometry = None;
            return;
        };

        encoder.copy_texture_to_buffer(
            TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: pos,
                aspect: TextureAspect::All,
            },
            TexelCopyBufferInfo {
                buffer: &self.staging,
                layout: Default::default(),
            },
            Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        self.state = StateMachine::Copied;
    }

    fn download(&mut self, app: &mut App) {
        let view = self.staging.slice(..).get_mapped_range();
        let [model, face] = bytemuck::pod_read_unaligned::<[u32; 2]>(&view);
        drop(view);
        self.staging.unmap();

        let support = model >> 31 != 0;
        let model = ModelId::from_raw(model & MODEL_ID_MASK);
        app.state.hovered_geometry = app.project.model(model).is_some().then_some(GeometryHit {
            model,
            support,

            face,
        });
        self.state = StateMachine::None;
    }
}

pub fn pick_selection(app: &mut App, area: Bounds2D) {
    if area.is_empty() {
        return;
    }

    let mut renderer = app.render_state.renderer.write();
    let mut resources = renderer
        .callback_resources
        .get_mut::<WorkspaceRenderResources>();
    let resources = resources.as_mut().unwrap();
    let Some(viewport) = &mut resources.model.viewport else {
        return;
    };

    let texture = viewport.model_target.texture();
    let (Some(start), Some(end)) = (
        uv_to_origin(texture.size(), area.min.map(|x| x.saturate())),
        uv_to_origin(texture.size(), area.max.map(|x| x.saturate())),
    ) else {
        return;
    };

    let extent = Extent3d {
        width: end.x - start.x,
        height: end.y - start.y,
        depth_or_array_layers: 1,
    };
    let padded_width = (extent.width * 8).next_multiple_of(COPY_BYTES_PER_ROW_ALIGNMENT);

    let device = &app.render_state.device;
    let staging = device.create_buffer(&BufferDescriptor {
        label: Some("Area Selection Staging"),
        size: (padded_width * extent.height) as u64,
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: start,
            aspect: TextureAspect::All,
        },
        TexelCopyBufferInfo {
            buffer: &staging,
            layout: TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_width),
                rows_per_image: None,
            },
        },
        extent,
    );

    app.render_state.queue.submit(iter::once(encoder.finish()));

    let ready = Arc::new(AtomicBool::new(false));
    staging.slice(..).map_async(
        MapMode::Read,
        clone!([ready], move |_| ready.store(true, Ordering::Relaxed)),
    );

    resources.model.picker.areas.push((staging, extent, ready));
}

fn uv_to_origin(size: Extent3d, uv: Vector2<f32>) -> Option<Origin3d> {
    let pos = Origin3d {
        x: (uv.x * size.width as f32).round() as u32,
        y: (uv.y * size.height as f32).round() as u32,
        z: 0,
    };

    let in_bounds = (0..size.width).contains(&pos.x) && (0..size.height).contains(&pos.y);
    in_bounds.then_some(pos)
}

fn download_area(app: &mut App, buffer: &Buffer, extent: &Extent3d) {
    let view = buffer.slice(..).get_mapped_range();
    let data = bytemuck::cast_slice::<_, [u32; 2]>(&view);

    let padded_width = (extent.width * 8).next_multiple_of(COPY_BYTES_PER_ROW_ALIGNMENT);
    for (x, y) in (0..extent.width).cartesian_product(0..extent.height) {
        let idx = (y * padded_width / 8) + x;
        let [model_raw, face] = data[idx as usize];
        if model_raw != u32::MAX {
            let support = model_raw >> 31 != 0;
            let model_id = ModelId::from_raw(model_raw & MODEL_ID_MASK);

            match app.state.tool {
                Tool::Select if !support => app.state.selected.select_model(model_id),
                Tool::Support if support => {
                    if let Some(model) = app.project.model(model_id)
                        && let Some(support_id) = model.supports.support_for_face(face)
                    {
                        (app.state.selected_supports).select_support(model_id, support_id);
                    }
                }
                _ => {}
            }
        }
    }
}
