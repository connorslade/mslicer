use std::{
    mem,
    sync::{Arc, RwLock},
};

use egui_wgpu::CallbackTrait;
use nalgebra::Matrix4;
use wgpu::{BindGroup, Buffer, IndexFormat, RenderPipeline, COPY_BUFFER_ALIGNMENT};

use crate::render::{RenderedMesh, RenderedMeshBuffers};

pub mod camera;
pub mod render;

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum RenderStyle {
    Normals,
    Rended,
}

pub struct WorkspaceRenderResources {
    pub render_pipeline: RenderPipeline,
    pub uniform_buffer: Buffer,
    pub bind_group: BindGroup,
}

pub struct WorkspaceRenderCallback {
    pub transform: Matrix4<f32>,
    pub modals: Arc<RwLock<Vec<RenderedMesh>>>,
    pub render_style: RenderStyle,
}

impl CallbackTrait for WorkspaceRenderCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources = callback_resources
            .get::<WorkspaceRenderResources>()
            .unwrap();

        let mut to_generate = Vec::new();
        for (idx, modal) in self.modals.read().unwrap().iter().enumerate() {
            if modal.try_get_buffers().is_none() {
                to_generate.push(idx);
            }
        }

        if !to_generate.is_empty() {
            let mut meshes = self.modals.write().unwrap();
            for idx in to_generate {
                meshes[idx].get_buffers(device);
            }
        }

        // todo: only do on change
        queue.write_buffer(&resources.uniform_buffer, 0, &self.to_wgsl());

        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut egui_wgpu::wgpu::RenderPass<'a>,
        callback_resources: &'a egui_wgpu::CallbackResources,
    ) {
        let resources = callback_resources
            .get::<WorkspaceRenderResources>()
            .unwrap();

        render_pass.set_pipeline(&resources.render_pipeline);
        render_pass.set_bind_group(0, &resources.bind_group, &[]);

        let modals = self.modals.read().unwrap();
        for modal in modals.iter() {
            // SAFETY: im really tired and i dont care anymore
            let buffers: &RenderedMeshBuffers =
                unsafe { &*(modal.try_get_buffers().unwrap() as *const _) };
            render_pass.set_vertex_buffer(0, buffers.vertex_buffer.slice(..));
            render_pass.set_index_buffer(buffers.index_buffer.slice(..), IndexFormat::Uint32);
            render_pass.draw_indexed(0..(3 * modal.mesh.faces.len() as u32), 0, 0..1);
        }
    }
}

impl WorkspaceRenderCallback {
    const PADDED_SIZE: u64 = ((mem::size_of::<Self>() as u64 + COPY_BUFFER_ALIGNMENT - 1)
        / COPY_BUFFER_ALIGNMENT)
        * COPY_BUFFER_ALIGNMENT;

    fn to_wgsl(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::PADDED_SIZE as usize);
        out.extend_from_slice(bytemuck::cast_slice(&self.transform.as_slice()));
        out.push(self.render_style as u8);
        out.resize(Self::PADDED_SIZE as usize, 0);
        out
    }
}

impl RenderStyle {
    pub fn name(&self) -> &'static str {
        match self {
            RenderStyle::Normals => "Normals",
            RenderStyle::Rended => "Rended",
        }
    }
}
