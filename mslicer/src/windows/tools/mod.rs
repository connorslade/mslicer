use tools::{
    exposure_test::ExposureTest, internal_exposure_test::InternalExposureTest,
    printed_circuit_board::PrintedCircuitBoard,
};

pub mod exposure_test;
pub mod internal_exposure_test;
pub mod printed_circuit_board;

#[derive(Default)]
pub struct Tools {
    exposure_test: ExposureTest,
    internal_exposure_test: InternalExposureTest,
    printed_circuit_board: PrintedCircuitBoard,
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

        let mut config = $app.project.slice_config.clone();
        let operation = SliceOperation::new(Progress::new(), CombinedProgress::new());
        operation.add_preview_image(RgbaImage::new(128, 128)); // blank preview image
        let tool = $tool.clone();
        tool.slice_config(&mut config);

        std::thread::spawn(clone!([operation], move || {
            let layers = tool.generate(&config, &operation.progress);
            operation.add_raster_result(config, layers);
        }));
        $app.slice_operation.replace(operation);
        $app.panels
            .focus_tab(Tab::SlicePreview, Vector2::new(700.0, 400.0));
    }};
}
