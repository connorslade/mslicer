mod consts;
pub mod model;
pub mod slice_preview;
pub mod solid_line;
pub mod target_point;

#[macro_export]
macro_rules! include_shader {
    ($shader:literal) => {
        ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(include_str!(concat!("../shaders/", $shader)).into()),
        }
    };
}
