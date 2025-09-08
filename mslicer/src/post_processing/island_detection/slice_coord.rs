use std::{fmt::Display, hash::Hash, ops::Add};

use common::config::SliceConfig;
use itertools::Itertools;

fn neighborhood(
    width: i64,
    height: i64,
    radius: i16,
    px: i64,
    py: i64,
) -> impl Iterator<Item = usize> {
    let r = radius as i64;
    (-r..=r)
        .cartesian_product(-r..=r)
        .filter_map(move |(y, x)| {
            let nx = px + x;
            let ny = py + y;
            if nx >= 0 && nx < width && ny >= 0 && ny < height {
                Some((ny * width + px) as usize)
            } else {
                None
            }
        })
}

#[derive(Clone, Copy, Debug)]
pub struct SliceCoord<'a> {
    pub cfg: &'a SliceConfig,
    pub idx: usize,
}

impl<'a> SliceCoord<'a> {
    pub fn new(cfg: &'a SliceConfig) -> Self {
        Self { cfg, idx: 0 }
    }

    #[inline]
    pub fn x(&self) -> i32 {
        (self.idx % self.cfg.platform_resolution[0] as usize) as i32
    }

    #[inline]
    pub fn y(&self) -> i32 {
        (self.idx / self.cfg.platform_resolution[0] as usize) as i32
    }

    pub fn neighborhood(&self, radius: i16) -> impl Iterator<Item = usize> {
        neighborhood(
            self.cfg.platform_resolution.x as i64,
            self.cfg.platform_resolution.y as i64,
            radius,
            self.x() as i64,
            self.y() as i64,
        )
    }
}

impl Eq for SliceCoord<'_> {}

impl PartialEq for SliceCoord<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx && self.cfg.platform_resolution == other.cfg.platform_resolution
    }
}

impl Hash for SliceCoord<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.idx);
    }
}

impl Add<usize> for SliceCoord<'_> {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self {
            cfg: self.cfg,
            idx: self.idx + rhs,
        }
    }
}

impl<'a> From<(&'a SliceConfig, usize)> for SliceCoord<'a> {
    fn from(value: (&'a SliceConfig, usize)) -> Self {
        Self {
            cfg: value.0,
            idx: value.1,
        }
    }
}

impl Display for SliceCoord<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x(), self.y())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector2;
    use proptest::prelude::*;
    use rayon::prelude::*;

    prop_compose! {
        fn arb_slice_config()(args in (1_u32..200, 1_u32..200)) -> SliceConfig {
            let mut cfg = SliceConfig::default();
            let (w, h) = args;
            cfg.platform_resolution = Vector2::new(w, h);
            cfg
        }
    }

    proptest! {
        #[test]
        fn test_neighborhoods(radius in 0_i16..10, cfg in arb_slice_config()) {
            // incomplete neighborhoods only occur within radius pixels of the border
            let (w, h) = (cfg.platform_resolution.x as i64, cfg.platform_resolution.y as i64);
            let area = w * h;
            let rr = 2 * radius as i64;
            let complete_area = (w - rr.min(w)) * (h - rr.min(h));
            let expected_num_of_incomplete_neighborhoods = area - complete_area;
            let fullsz = (radius as i64 * 2 + 1) * (radius as i64 * 2 + 1);
            println!("{} x {} @ radius {} has {} expected incomplete neighborhoods {}
                (0, 0) neighborhood has size {}",
                cfg.platform_resolution.x,
                cfg.platform_resolution.y,
                radius,
                expected_num_of_incomplete_neighborhoods,
                fullsz,
                SliceCoord::from((&cfg, 0usize)).neighborhood(radius).collect::<Vec<_>>().len());

            let num = std::sync::atomic::AtomicI64::new(0);
            (0..cfg.platform_resolution.y)
                .into_par_iter()
                .for_each(|y| {
                for x in 0..cfg.platform_resolution.x {
                    let ns = SliceCoord::from((&cfg, (y * cfg.platform_resolution.x + x) as usize))
                        .neighborhood(radius)
                        .collect::<Vec<_>>()
                        .len();
                    if (ns as i64) < fullsz {
                        num.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    }
                }});
            let total = num.into_inner();
            assert_eq!(total, expected_num_of_incomplete_neighborhoods,
                "{} x {} @ radius {} has {} incomplete neighborhoods, expected: {}",
                cfg.platform_resolution.x,
                cfg.platform_resolution.y,
                radius,
                total,
                expected_num_of_incomplete_neighborhoods);
        }
    }
}
