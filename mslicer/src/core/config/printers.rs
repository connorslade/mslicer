use std::borrow::Cow;

use common::{slice::SliceConfig, units::Milimeters};
use nalgebra::{Vector2, Vector3};
use serde::{Deserialize, Serialize};

use crate::{core::config::Config, core::selected::SelectedPrinter};

#[derive(Clone, Serialize, Deserialize)]
pub struct Printer {
    pub name: Cow<'static, str>,
    pub resolution: Vector2<u32>,
    pub size: Vector3<Milimeters>,
    pub validated: bool,
}

impl Printer {
    pub const fn new(name: &'static str, [rx, ry]: [u32; 2], [sx, sy, sz]: [f32; 3]) -> Self {
        Self {
            name: Cow::Borrowed(name),
            resolution: Vector2::new(rx, ry),
            size: Vector3::new(
                Milimeters::new(sx),
                Milimeters::new(sy),
                Milimeters::new(sz),
            ),
            validated: false,
        }
    }

    pub const fn validated(mut self) -> Self {
        self.validated = true;
        self
    }
}

impl Default for Printer {
    fn default() -> Self {
        Self {
            name: Cow::Owned("New Printer".into()),
            resolution: Vector2::new(10_000, 5_000),
            size: Vector3::repeat(100.0).map(Milimeters::new),
            validated: false,
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

#[rustfmt::skip]
/// Many of these constants were taken from ChituBox v2.3.0 with
/// `scripts/scrape-chitu-profiles.ysh`. Validated printers are known to work
/// with mslicer.
pub const DEFAULT_PRINTERS: &[(&str, &[Printer])] = &[
    ("Elegoo", &[
        Printer::new("Jupiter Se",         [ 5448, 3064], [277.848, 156.264, 300.0]),
        Printer::new("Jupiter",            [ 5448, 3064], [277.848, 156.264, 300.0]),
        Printer::new("Mars 2 Pro",         [ 1620, 2560], [ 82.620, 130.560, 160.0]).validated(),
        Printer::new("Mars 2",             [ 1620, 2560], [ 82.620, 130.560, 150.0]),
        Printer::new("Mars 3 Pro",         [ 4098, 2560], [143.430,  89.600, 175.0]),
        Printer::new("Mars 3",             [ 4098, 2560], [143.430,  89.600, 175.0]),
        Printer::new("Mars 4 Dlp",         [ 2560, 1440], [132.800,  74.700, 150.0]),
        Printer::new("Mars 4 Max",         [ 5760, 3600], [195.840, 122.400, 150.0]),
        Printer::new("Mars 4 Ultra",       [ 8520, 4320], [153.360,  77.760, 165.0]),
        Printer::new("Mars 4",             [ 8520, 4320], [153.360,  77.760, 175.0]),
        Printer::new("Mars 5 Ultra",       [ 8520, 4320], [153.360,  77.760, 165.0]).validated(),
        Printer::new("Mars 5",             [ 4098, 2560], [143.430,  89.600, 150.0]),
        Printer::new("Mars C",             [ 1440, 2560], [ 68.040, 120.960, 150.0]),
        Printer::new("Mars Pro",           [ 1440, 2560], [ 68.040, 120.960, 150.0]),
        Printer::new("Mars",               [ 1440, 2560], [ 68.040, 120.960, 150.0]),
        Printer::new("Saturn 2",           [ 7680, 4320], [218.880, 123.120, 250.0]),
        Printer::new("Saturn 3 Ultra",     [11520, 5120], [218.880, 122.880, 260.0]).validated(),
        Printer::new("Saturn 3",           [11520, 5120], [218.880, 122.880, 250.0]),
        Printer::new("Saturn 4 Ultra 16K", [15120, 6230], [211.680, 118.370, 220.0]).validated(),
        Printer::new("Saturn 4 Ultra",     [11520, 5120], [218.880, 122.880, 220.0]),
        Printer::new("Saturn 4",           [11520, 5120], [218.880, 122.880, 220.0]),
        Printer::new("Saturn 8K",          [ 7680, 4320], [218.880, 123.120, 210.0]),
        Printer::new("Saturn S",           [ 4098, 2560], [196.704, 122.880, 210.0]),
        Printer::new("Saturn",             [ 3840, 2400], [192.000, 120.000, 200.0]),
    ]),
    ("Phrozen", &[
        Printer::new("Sonic 4K",          [ 3840, 2160], [134.40,  75.60, 200.0]),
        Printer::new("Sonic Mega 8K S",   [ 7680, 4320], [330.24, 185.76, 300.0]),
        Printer::new("Sonic Mega 8K V2",  [ 7680, 4320], [330.24, 185.76, 400.0]),
        Printer::new("Sonic Mega 8K",     [ 7680, 4320], [330.24, 185.76, 400.0]),
        Printer::new("Sonic Mighty 12K",  [11520, 5120], [218.88, 123.12, 235.0]),
        Printer::new("Sonic Mighty 4K",   [ 3840, 2400], [199.68, 124.80, 220.0]),
        Printer::new("Sonic Mighty 8K",   [ 7680, 4320], [218.88, 123.12, 235.0]),
        Printer::new("Sonic Mighty Revo", [13320, 5120], [223.78, 126.98, 235.0]),
        Printer::new("Sonic Mini 4K",     [ 3840, 2160], [134.40,  75.60, 130.0]),
        Printer::new("Sonic Mini 8K",     [ 7500, 3240], [165.00,  71.28, 180.0]),
    ]),
    ("Concepts3D", &[
        Printer::new("Athena II", [15120, 6230], [211.68, 118.37, 235.0]).validated(),
    ]),
    ("ApexMaker", &[
        Printer::new("X1 mini", [13320, 5120], [223.776, 126.976, 200.0]),
        Printer::new("X1",      [ 7680, 4320], [353.280, 198.720, 400.0]),
    ]),
    ("Creality", &[
        Printer::new("LD-002H",    [1620, 2560], [ 82.620, 130.560, 160.0]),
        Printer::new("LD-002R",    [1440, 2560], [ 68.040, 120.960, 160.0]),
        Printer::new("LD-006",     [3840, 2400], [192.000, 120.000, 250.0]),
        Printer::new("HALOT-MAGE", [7680, 4320], [228.096, 128.304, 230.0]),
    ]),
    ("Emake3D", &[
        Printer::new("Megalabs 1", [ 3840, 2160], [533.00, 299.8125, 550.0]),
        Printer::new("Stellar 1",  [13320, 5120], [223.78, 126.9800, 200.0]),
    ]),
    ("EPAX", &[
        Printer::new("DX1 PRO",         [ 4098, 2560], [143.43,  89.600, 120.0]),
        Printer::new("DX10 PRO 5K",     [ 4920, 2880], [221.40, 129.600, 120.0]),
        Printer::new("DX10 PRO 8K/8KW", [ 7680, 4320], [218.88, 123.120, 120.0]),
        Printer::new("E10 14K",         [ 7680, 4320], [218.88, 123.120, 250.0]),
        Printer::new("E10 14KW",        [13320, 5120], [223.78, 126.980, 240.0]),
        Printer::new("E10 4K",          [ 3840, 2400], [192.00, 120.000, 250.0]),
        Printer::new("E10 5K",          [ 4920, 2880], [221.40, 129.600, 250.0]),
        Printer::new("E10 8K",          [ 7680, 4320], [218.88, 123.120, 250.0]),
        Printer::new("E6",              [ 1620, 2560], [ 81.00, 128.000, 155.0]),
        Printer::new("X1-4K",           [ 2160, 3840], [ 68.04, 120.960, 155.0]),
        Printer::new("X1-4KS",          [ 4098, 2560], [143.43,  89.600, 155.0]),
        Printer::new("X1",              [ 1440, 2560], [ 68.04, 120.960, 155.0]),
        Printer::new("X10 14KW",        [13320, 5120], [223.78, 126.980, 250.0]),
        Printer::new("X10 2K Color",    [ 1600, 2560], [135.36, 216.576, 250.0]),
        Printer::new("X10 4K",          [ 3840, 2400], [192.00, 120.000, 250.0]),
        Printer::new("X10 5K",          [ 4920, 2880], [221.40, 129.600, 250.0]),
        Printer::new("X10 8K/8kW",      [ 7680, 4320], [218.88, 123.120, 250.0]),
        Printer::new("X133 4K",         [ 3840, 2160], [293.76, 165.240, 400.0]),
        Printer::new("X133 6K",         [ 5760, 3240], [288.00, 162.000, 400.0]),
        Printer::new("X133 7K",         [ 6480, 3600], [298.08, 165.600, 400.0]),
        Printer::new("X156 4K Color",   [ 3840, 2160], [345.60, 194.400, 400.0]),
        Printer::new("X160 8K",         [ 7680, 4320], [353.28, 198.720, 400.0]),
        Printer::new("X1K",             [ 1620, 2560], [ 82.62, 130.560, 155.0]),
    ]),
    ("HIFUN", &[
        Printer::new("HF-L1-9K",      [ 8520, 4320], [153.36,  77.76, 150.0]),
        Printer::new("HF-L3-14K PRO", [13320, 5120], [223.78, 126.98, 240.0]),
        Printer::new("HF-L3-14K",     [13320, 5120], [223.78, 126.98, 250.0]),
        Printer::new("HF-L5-7K",      [ 6480, 3600], [298.08, 165.60, 350.0]),
        Printer::new("HF-L6-8K",      [ 7680, 4320], [353.28, 198.72, 400.0]),
    ]),
    ("HITRY", &[
        Printer::new("Rocket 1 Pro", [3840, 2400], [145.000,  90.625, 150.0]),
        Printer::new("Rocket 1",     [1620, 2560], [ 86.505, 136.700, 150.0]),
    ]),
    ("LYNcase", &[
        Printer::new("LYN cast LY-01", [2560, 1440], [134.4, 75.6, 165.0]),
    ]),
    ("Magforms", &[
        Printer::new("P13", [5448, 3064], [277.848, 156.264, 395.0]),
    ]),
    ("NOVA3D", &[
        Printer::new("Whale2 Pro",       [ 7680, 4320], [228.096, 128.304, 250.0]),
        Printer::new("Whale3 Pro",       [ 7680, 4320], [228.096, 128.304, 260.0]),
        Printer::new("Whale3 SE",        [ 7680, 4320], [228.096, 128.304, 260.0]),
        Printer::new("Whale3 Ultra-14k", [13320, 5120], [223.780, 126.980, 260.0]),
        Printer::new("Whale4 16K",       [15120, 6230], [211.680, 118.370, 220.0]),
        Printer::new("Whale4",           [15120, 6230], [211.680, 118.370, 220.0]),
        Printer::new("Whale3 Super-14k", [13320, 5120], [223.780, 126.980, 260.0]),
    ]),
    ("Newbie Box", &[
        Printer::new("Kylin Two",   [ 7680, 4320], [228.096, 128.304, 350.0]),
        Printer::new("Phoenix One", [13320, 5120], [223.780, 126.980, 260.0]),
    ]),
    ("Peopoly", &[
        Printer::new("Phenom L",     [3840, 2160], [345.600, 194.400, 400.0]),
        Printer::new("Phenom Noir",  [3840, 2160], [293.760, 165.240, 400.0]),
        Printer::new("Phenom Prime", [5448, 3064], [277.848, 156.264, 400.0]),
        Printer::new("Phenom XXL",   [3840, 2160], [527.040, 296.460, 550.0]),
        Printer::new("Phenom",       [3840, 2160], [276.480, 155.520, 400.0]),
    ]),
    ("QIDI", &[
        Printer::new("6.08 mono",     [1620, 2560], [ 82.62, 130.560, 150.0]),
        Printer::new("S-box",         [1600, 2560], [135.36, 216.576, 200.0]),
        Printer::new("Shadow5.5s",    [1440, 2560], [ 68.04, 120.960, 150.0]),
        Printer::new("Shadow6.0 Pro", [1440, 2560], [ 74.52, 132.480, 150.0]),
        Printer::new("i-box mono",    [3840, 2400], [192.00, 120.000, 200.0]),
    ]),
    ("UniFormation", &[
        Printer::new("GK3 Pro",   [15120, 6230], [211.68, 118.37, 240.0]),
        Printer::new("GK3 Ultra", [15120, 6230], [302.40, 161.98, 300.0]),
        Printer::new("GK3",       [15120, 6230], [211.68, 118.37, 240.0]),
        Printer::new("GKtwo",     [ 7680, 4320], [228.10, 128.30, 245.0]),
    ]),
    ("WanHao", &[
        Printer::new("CGR MINI MONO", [1620, 2560], [82.62, 130.56, 200.0]),
        Printer::new("CGR MONO",      [3840, 2400], [192.0, 120.0,  180.0]),
    ])
];
