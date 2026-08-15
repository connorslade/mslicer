use std::borrow::Cow;

use common::{slice::SliceConfig, units::Milimeters};
use nalgebra::{Vector2, Vector3};
use serde::{Deserialize, Serialize};

use crate::{app::config::Config, ui::state::SelectedPrinter};

#[rustfmt::skip]
pub const DEFAULT_PRINTERS: &[(&str, &[PrinterProperties])] = &[
    ("Elegoo", &[
        PrinterProperties::new("Saturn 3",              [11_520, 5_120], [218.88,  122.88,  250.0]),
        PrinterProperties::new("Saturn 3 Ultra",        [11_520, 5_120], [218.88,  122.904, 260.0]),
        PrinterProperties::new("Saturn 4",              [11_520, 5_120], [218.88,  122.88,  220.0]),
        PrinterProperties::new("Saturn 4 Ultra",        [11_520, 5_120], [218.88,  122.88,  220.0]),
        PrinterProperties::new("Saturn 4 Ultra 16K",    [15_120, 6_230], [211.68,  118.37,  220.0]),
        PrinterProperties::new("Jupiter SE",            [5_448,  3_064], [277.848, 156.264, 300.0]),
        PrinterProperties::new("Jupiter 2",             [15_120, 6_230], [302.0,   162.0,   300.0]),
        PrinterProperties::new("Mars 5",                [4_098,  2_560], [143.43,  89.6,    150.0]),
        PrinterProperties::new("Mars 5 Ultra",          [8_520,  4_320], [153.36,  77.76,   165.0]),
        PrinterProperties::new("Mars 4",                [8_520,  4_320], [153.36,  77.76,   175.0]),
        PrinterProperties::new("Mars 4 Ultra",          [8_520,  4_320], [153.36,  77.76,   165.0]),
    ]),
    ("Phrozen", &[
        PrinterProperties::new("Sonic Mini 4K",         [3_840,  2_160], [134.40,  75.600,  130.0]),
        PrinterProperties::new("Sonic Mini 8K",         [7_500,  3_240], [165.00,  71.280,  180.0]),
        PrinterProperties::new("Sonic Mega 8K",         [7_680,  4_320], [330.24,  185.76,  400.0]),
        PrinterProperties::new("Sonic Mega 8K V2",      [7_680,  4_320], [330.24,  185.76,  400.0]),
        PrinterProperties::new("Sonic Mighty 8K",       [7_680,  4_320], [218.88,  123.12,  235.0]),
        PrinterProperties::new("Sonic Mighty 12K",      [11_520, 5_120], [218.88,  123.12,  235.0]),
        PrinterProperties::new("Sonic Mighty Revo",     [13_320, 5_120], [223.78,  126.98,  235.0]),
        PrinterProperties::new("Sonic Mighty Revo 16K", [15_120, 6_230], [211.68,  118.37,  235.0]), // verify!
    ])
];

#[derive(Clone, Serialize, Deserialize)]
pub struct PrinterProperties {
    pub name: Cow<'static, str>,
    pub resolution: Vector2<u32>,
    pub size: Vector3<Milimeters>,
}

impl PrinterProperties {
    pub const fn new(name: &'static str, [rx, ry]: [u32; 2], [sx, sy, sz]: [f32; 3]) -> Self {
        Self {
            name: Cow::Borrowed(name),
            resolution: Vector2::new(rx, ry),
            size: Vector3::new(
                Milimeters::new(sx),
                Milimeters::new(sy),
                Milimeters::new(sz),
            ),
        }
    }
}

pub fn selected_printer(config: &Config, slice_config: &SliceConfig) -> SelectedPrinter {
    for (i, printer) in config.printers.iter().enumerate() {
        if printer.resolution == slice_config.platform_resolution
            && printer.size == slice_config.platform_size
        {
            return SelectedPrinter::Custom(i);
        }
    }

    for (i, brand) in DEFAULT_PRINTERS.iter().enumerate() {
        for (j, printer) in brand.1.iter().enumerate() {
            if printer.resolution == slice_config.platform_resolution
                && printer.size == slice_config.platform_size
            {
                return SelectedPrinter::Preset(i, j);
            }
        }
    }

    SelectedPrinter::Project
}
