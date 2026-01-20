use common::{
    progress::Progress,
    serde::{Deserializer, Serializer},
};
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
    pub fn post_slice(&self, file: &mut FormatSliceFile, progress: Progress) {
        if !self.enabled {
            return;
        }

        progress.set_total(file.info().layers as u64);
        file.iter_mut_layers().par_bridge().for_each(|mut layer| {
            progress.add_complete(1);
            let (width, height) = (layer.width(), layer.height());
            let mut image =
                BlurImageMut::borrow(&mut layer, width, height, FastBlurChannels::Plane);
            libblur::fast_gaussian_next(
                &mut image,
                AnisotropicRadius::new(self.radius as u32),
                ThreadingPolicy::Single,
                EdgeMode2D::new(EdgeMode::Clamp),
            )
            .unwrap();
        });

        progress.set_finished();
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

impl AntiAlias {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_bool(self.enabled);
        ser.write_f32_be(self.radius);
    }

    pub fn deserialize<T: Deserializer>(des: &mut T) -> Self {
        Self {
            enabled: des.read_bool(),
            radius: des.read_f32_be(),
        }
    }
}
