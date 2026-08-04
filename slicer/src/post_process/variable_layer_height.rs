use common::{
    container::rle::difference,
    progress::Progress,
    serde::{Deserializer, Serializer},
    slice::{Layer, SliceConfig},
    units::{Milimeter, Second, Seconds},
};
use tracing::info;

#[derive(Clone)]
pub struct VariableLayerHeight {
    pub enabled: bool,

    /// Maximum allowed value deviation (value/mm²)
    pub threshold: f32,
    /// Max number of layers to be merged
    pub max_layers: u8,
    /// Exposure time added per layer (sec)
    pub exposure: Seconds,
}

impl VariableLayerHeight {
    pub fn post_slice(&self, config: &SliceConfig, layers: &mut Vec<Layer>, progress: Progress) {
        if !self.enabled {
            return;
        }

        progress.set_total(layers.len() as _);

        let px_per_mm = (config.platform_resolution.cast::<f32>())
            .component_div(&config.platform_size.xy().map(|x| x.get::<Milimeter>()));
        let px2_per_mm2 = px_per_mm.x * px_per_mm.y;
        let threshold = self.threshold * px2_per_mm2 * 255.0;

        let mut i = 0;
        let mut merged = 0;
        let mut counter = 0;

        while i < layers.len() - 1 {
            // If next layer is identical, remove and merge into current
            if merged < self.max_layers
                && (difference(&layers[i].data, &layers[i + 1].data) as f32) < threshold
            {
                let old = layers.remove(i + 1);
                let new = &mut layers[i];

                new.exposure.exposure_time = new.exposure.exposure_time + self.exposure;
                new.height = old.height;

                merged += 1;
                counter += 1;
            } else {
                merged = 0;
                i += 1;
            }
        }

        info!("Merged {counter} layers");
        progress.set_finished();
    }
}

impl Default for VariableLayerHeight {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold: 0.5,
            max_layers: 2,
            exposure: Seconds::new(0.5),
        }
    }
}

impl VariableLayerHeight {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_bool(self.enabled);
        ser.write_f32_be(self.threshold);
        ser.write_u8(self.max_layers);
        ser.write_f32_be(self.exposure.get::<Second>());
    }

    pub fn deserialize<T: Deserializer>(des: &mut T, version: u16) -> Self {
        Self {
            enabled: des.read_bool(),
            threshold: if version < 11 { 0.5 } else { des.read_f32_be() },
            max_layers: des.read_u8(),
            exposure: Seconds::new(des.read_f32_be()),
        }
    }
}
