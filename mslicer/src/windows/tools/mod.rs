use slicer::tools::{exposure_test::ExposureTest, internal_exposure_test::InternalExposureTest};

pub mod exposure_test;
pub mod internal_exposure_test;

#[derive(Default)]
pub struct Tools {
    exposure_test: ExposureTest,
    internal_exposure_test: InternalExposureTest,
}

// i couldn't get lifetimes working to do this with a function... so
#[macro_export]
macro_rules! generator_tool {
    ($app:expr, $tool:expr) => {{
        use clone_macro::clone;
        use image::RgbaImage;
        use nalgebra::Vector2;

        use common::progress::{CombinedProgress, Progress};
        use $crate::{app::slice_operation::SliceOperation, windows::Tab};

        let config = $app.project.slice_config.clone();
        let operation = SliceOperation::new(Progress::new(), CombinedProgress::new());
        operation.add_preview_image(RgbaImage::new(128, 128)); // blank preview image
        let tool = $tool.clone();

        std::thread::spawn(clone!([operation], move || {
            let layers = tool.generate(&config, &operation.progress);
            operation.add_raster_result(config, layers);
        }));
        $app.slice_operation.replace(operation);
        $app.panels
            .focus_tab(Tab::SliceOperation, Vector2::new(700.0, 400.0));
    }};
}
