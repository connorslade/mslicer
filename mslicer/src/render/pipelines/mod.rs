mod consts;
pub mod line;
pub mod model;
pub mod point;
pub mod slice_preview;

#[macro_export]
macro_rules! include_shader {
    ($shader:literal) => {
        wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!(concat!("../shaders/", $shader)).into()),
        }
    };
}
