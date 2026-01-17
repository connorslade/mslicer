use libblur::{
    AnisotropicRadius, BlurImageMut, EdgeMode, EdgeMode2D, FastBlurChannels, ThreadingPolicy,
};
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::format::FormatSliceFile;

#[derive(Clone, Serialize, Deserialize)]
pub struct AntiAlias {
    pub enabled: bool,
    pub radius: f32,
}

impl AntiAlias {
    pub fn post_slice(&self, file: &mut FormatSliceFile) {
        if !self.enabled {
            return;
        }

        file.iter_mut_layers().par_bridge().for_each(|mut layer| {
            let (width, height) = (layer.width(), layer.height());
            let mut image =
                BlurImageMut::borrow(&mut layer, width, height, FastBlurChannels::Plane);
            libblur::fast_gaussian_next(
                &mut image,
                AnisotropicRadius::new(self.radius as u32),
                ThreadingPolicy::Adaptive,
                EdgeMode2D::new(EdgeMode::Clamp),
            )
            .unwrap();
        });
    }
}

impl Default for AntiAlias {
    fn default() -> Self {
        Self {
            enabled: false,
            radius: 1.0,
        }
    }
}
