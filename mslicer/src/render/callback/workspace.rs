use std::{iter, sync::Arc};

use egui::PaintCallbackInfo;
use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};
use nalgebra::{Matrix4, Vector3};
use parking_lot::RwLock;
use slicer::mesh::Mesh;
use wgpu::{
    Color, CommandBuffer, CommandEncoder, CommandEncoderDescriptor, Device, LoadOp, Operations,
    Queue, RenderPass, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, StoreOp, Texture, TextureFormat, TextureViewDescriptor,
};

use crate::{
    app::config::Config,
    render::{
        Gcx,
        camera::Camera,
        dispatch::{line::LineDispatch, point::PointDispatch},
        init_textures,
        model::Model,
        pipelines::{composite::CompositePipeline, model::ModelPipeline, support::SupportPipeline},
    },
};

pub struct WorkspaceRenderResources {
    pub texture: TextureFormat,
    pub composite: CompositePipeline,

    pub model: ModelPipeline,
    pub support: SupportPipeline,

    pub point: PointDispatch,
    pub solid_line: LineDispatch,
}

#[derive(Clone)]
pub struct WorkspaceRenderCallback {
    pub camera: Camera,
    pub transform: Matrix4<f32>,
    pub is_moving: bool,

    pub bed_size: Vector3<f32>,
    pub grid_size: f32,

    pub models: Arc<RwLock<Vec<Model>>>,
    pub config: Config,

    pub line_support_debug: Vec<[Vector3<f32>; 2]>,
    pub support_model: Option<Mesh>,
    pub overhang_angle: Option<f32>,
}

pub struct WorkspaceRenderState {
    pub texture: Texture,
    pub resolved_texture: Texture,
    pub depth_texture: Texture,
}

impl CallbackTrait for WorkspaceRenderCallback {
    fn prepare(
        &self,
        device: &Device,
        queue: &Queue,
        screen: &ScreenDescriptor,
        _encoder: &mut CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<CommandBuffer> {
        let workspace = resources.get::<WorkspaceRenderResources>().unwrap();

        let [width, height] = screen.size_in_pixels;
        if !resources.contains::<WorkspaceRenderState>()
            || resources
                .get::<WorkspaceRenderState>()
                .unwrap()
                .depth_texture
                .size()
                .width
                != width
        {
            println!("new textures");
            let (texture, resolved_texture, depth_texture) =
                init_textures(device, workspace.texture, (width, height));

            resources.insert(WorkspaceRenderState {
                texture,
                resolved_texture,
                depth_texture,
            });
        };

        let state = resources.get::<WorkspaceRenderState>().unwrap();

        let texture_view = state.texture.create_view(&Default::default());
        let resolved_texture_view = state.resolved_texture.create_view(&Default::default());
        let depth_texture_view = state.depth_texture.create_view(&Default::default());

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: Some(&resolved_texture_view),
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &depth_texture_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        let workspace = resources.get_mut::<WorkspaceRenderResources>().unwrap();
        let gcx = Gcx { device, queue };
        workspace.model.prepare(&gcx, self);
        workspace.support.prepare(&gcx, self);
        workspace.solid_line.prepare(&gcx, self);
        workspace.point.prepare(&gcx, self);

        workspace.solid_line.paint(&mut render_pass);
        workspace.model.paint(&mut render_pass, self);
        workspace.point.paint(&mut render_pass);
        workspace.support.paint(&mut render_pass);
        drop(render_pass);

        queue.submit(iter::once(encoder.finish()));

        workspace.composite.prepare(&gcx, &resolved_texture_view);

        Vec::new()
    }

    fn paint(
        &self,
        _info: PaintCallbackInfo,
        render_pass: &mut RenderPass,
        callback_resources: &CallbackResources,
    ) {
        let resources = callback_resources
            .get::<WorkspaceRenderResources>()
            .unwrap();
        resources.composite.paint(render_pass);
    }
}
