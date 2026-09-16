use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use clone_macro::clone;
use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, CommandEncoder, Device, Extent3d, MapMode, Origin3d,
    PollType, TexelCopyBufferInfo, TexelCopyBufferLayout, TexelCopyTextureInfo, TextureAspect,
};

use crate::{
    app::App,
    project::model::ModelId,
    render::{Gcx, workspace::model::MultiStage},
    ui::state::GeometryHit,
};

pub struct ModelPicker {
    staging: Buffer,
    state: StateMachine,
}

enum StateMachine {
    None,
    Copied,
    Mapping(Arc<AtomicBool>),
}

impl ModelPicker {
    pub fn new(device: &Device) -> Self {
        let staging = device.create_buffer(&BufferDescriptor {
            label: None,
            size: size_of::<[u32; 2]>() as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Self {
            staging,
            state: StateMachine::None,
        }
    }

    pub fn update(
        &mut self,
        gcx: &Gcx,
        encoder: &mut CommandEncoder,
        multi: &MultiStage,
        app: &mut App,
    ) {
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
                let _ = gcx.device.poll(PollType::Poll);
                complete.load(Ordering::Relaxed).then(|| self.download(app));
            }
        }
    }

    fn copy(&mut self, encoder: &mut CommandEncoder, multi: &MultiStage, app: &mut App) {
        let texture = multi.model_target.texture();
        let size = texture.size();

        let uv = app.state.workspace.uv;
        if !(0.0..1.0).contains(&uv.x) || !(0.0..1.0).contains(&uv.y) {
            app.state.hovered_geometry = None;
            return;
        }

        let pos = Origin3d {
            x: (uv.x * size.width as f32).round() as u32,
            y: (uv.y * size.height as f32).round() as u32,
            z: 0,
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
                layout: TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: None,
                    rows_per_image: None,
                },
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

        app.state.hovered_geometry = (model != u32::MAX).then(|| GeometryHit {
            model: ModelId::from_raw(model),
            face,
        });
        self.state = StateMachine::None;
    }
}
