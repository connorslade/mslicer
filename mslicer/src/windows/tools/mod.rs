use slicer::tools::exposure_test::ExposureTest;

pub mod exposure_test;

#[derive(Default)]
pub struct Tools {
    exposure_test: ExposureTest,
}
