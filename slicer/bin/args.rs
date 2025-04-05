use std::{any::Any, path::PathBuf, str::FromStr};

use anyhow::{Context, Ok, Result};
use clap::{ArgMatches, Parser};
use common::{
    config::{ExposureConfig, SliceConfig},
    format::Format,
};
use nalgebra::{ArrayStorage, Const, Matrix, Scalar, Vector2, Vector3, U1};
use num_traits::Zero;

#[derive(Debug, Parser)]
/// mslicer command line interface.
pub struct Args {
    #[arg(long, default_value = "11520, 5120", value_parser = vector_value_parser::<u32, 2>, )]
    /// Resolution of the printer mask display in pixels.
    pub platform_resolution: Vector2<u32>,
    #[arg(long, default_value = "218.88, 122.904, 260.0", value_parser = vector_value_parser::<f32, 3>)]
    /// Size of the printer display / platform in mm.
    pub platform_size: Vector3<f32>,
    #[arg(long, default_value_t = 0.05)]
    /// Layer height in mm.
    pub layer_height: f32,
    #[arg(long, default_value_t = 3)]
    /// Number of 'first layers'. These are layers that obey the --first-
    /// exposure config flags.
    pub first_layers: u32,
    #[arg(long, default_value_t = 10)]
    /// Number of transition layers. These are layers that interpolate from the
    /// first layer config to the default config.
    pub transition_layers: u32,

    #[arg(long, default_value_t = 3.0)]
    /// Layer exposure time in seconds.
    pub exposure_time: f32,
    #[arg(long, default_value_t = 5.0)]
    /// Distance to lift the platform after exposing each regular layer, in mm.
    pub lift_distance: f32,
    #[arg(long, default_value_t = 65.0)]
    /// The speed to lift the platform after exposing each regular layer, in
    /// mm/min.
    pub lift_speed: f32,
    #[arg(long, default_value_t = 150.0)]
    /// The speed to retract (move down) the platform after exposing each
    /// regular layer, in mm/min.
    pub retract_speed: f32,

    #[arg(long, default_value_t = 30.0)]
    /// First layer exposure time in seconds.
    pub first_exposure_time: f32,
    #[arg(long, default_value_t = 5.0)]
    /// Distance to lift the platform after exposing each first layer, in mm.
    pub first_lift_distance: f32,
    #[arg(long, default_value_t = 65.0)]
    /// The speed to lift the platform after exposing each first layer, in
    /// mm/min.
    pub first_lift_speed: f32,
    #[arg(long, default_value_t = 150.0)]
    /// The speed to retract (move down) the platform after exposing each first
    /// layer, in mm/min.
    pub first_retract_speed: f32,

    #[arg(long)]
    /// Path to a preview image, will be scaled as needed.
    pub preview: Option<PathBuf>,

    #[command(flatten)]
    pub model: ModelArgs,

    /// File to save sliced result to. Currently only .goo files can be
    /// generated.
    pub output: PathBuf,
}

#[derive(clap::Args, Debug)]
#[group(required = true)]
pub struct ModelArgs {
    #[arg(long)]
    /// Path to a .stl or .obj file
    pub mesh: Vec<PathBuf>,

    #[arg(long, value_parser = vector_value_parser::<f32, 3>)]
    /// Location of the bottom center of model bounding box. The origin is the
    /// center of the build plate.
    pub position: Vec<Vector3<f32>>,

    #[arg(long, value_parser = vector_value_parser::<f32, 3>)]
    /// Rotation of the model in degrees, pitch, roll, yaw.
    pub rotation: Vec<Vector3<f32>>,

    #[arg(long, value_parser = vector_value_parser::<f32, 3>)]
    /// Scale of the model along the X, Y, and Z axes.
    pub scale: Vec<Vector3<f32>>,
}

#[derive(Debug)]
pub struct Model {
    pub path: PathBuf,
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub scale: Vector3<f32>,
}

impl Args {
    pub fn slice_config(&self) -> SliceConfig {
        SliceConfig {
            format: Format::Goo,
            platform_resolution: self.platform_resolution,
            platform_size: self.platform_size,
            slice_height: self.layer_height,
            exposure_config: ExposureConfig {
                exposure_time: self.exposure_time,
                lift_distance: self.lift_distance,
                lift_speed: self.lift_speed,
                retract_distance: self.lift_distance,
                retract_speed: self.retract_speed,
            },
            first_exposure_config: ExposureConfig {
                exposure_time: self.first_exposure_time,
                lift_distance: self.first_lift_distance,
                lift_speed: self.first_lift_speed,
                retract_distance: self.first_lift_distance,
                retract_speed: self.first_retract_speed,
            },
            first_layers: self.first_layers,
            transition_layers: self.transition_layers,
        }
    }

    pub fn mm_to_px(&self) -> Vector3<f32> {
        Vector3::new(
            self.platform_resolution.x as f32 / self.platform_size.x,
            self.platform_resolution.y as f32 / self.platform_size.y,
            1.0,
        )
    }
}

impl Model {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            ..Default::default()
        }
    }

    pub fn from_matches(matches: &ArgMatches) -> Vec<Self> {
        let mut meshes = matches
            .get_many::<PathBuf>("mesh")
            .expect("No meshes defined")
            .zip(matches.indices_of("mesh").unwrap())
            .map(|x| (x.1, Model::new(x.0.to_owned())))
            .collect::<Vec<_>>();

        fn model_parameter<T: Any + Clone + Send + Sync + 'static>(
            matches: &clap::ArgMatches,
            meshes: &mut [(usize, Model)],
            key: &str,
            value: impl Fn(&mut Model) -> &mut T,
        ) {
            let Some(instances) = matches.get_many::<T>(key) else {
                return;
            };

            for (instance, idx) in instances.zip(matches.indices_of(key).unwrap()) {
                let mesh = meshes
                    .iter_mut()
                    .rfind(|x| idx > x.0)
                    .expect("Mesh parameter before mesh");
                *value(&mut mesh.1) = instance.to_owned();
            }
        }

        model_parameter(matches, &mut meshes, "scale", |mesh| &mut mesh.scale);
        model_parameter(matches, &mut meshes, "rotation", |mesh| &mut mesh.rotation);
        model_parameter(matches, &mut meshes, "position", |mesh| &mut mesh.position);

        meshes.into_iter().map(|x| x.1).collect()
    }
}

impl Default for Model {
    fn default() -> Self {
        Self {
            path: PathBuf::default(),
            position: Vector3::zeros(),
            rotation: Vector3::zeros(),
            scale: Vector3::repeat(1.0),
        }
    }
}

fn vector_value_parser<T, const N: usize>(
    raw: &str,
) -> Result<Matrix<T, Const<N>, U1, ArrayStorage<T, N, 1>>>
where
    T: FromStr + Scalar + Zero,
    T::Err: Send + Sync + std::error::Error,
{
    let mut vec = Matrix::<T, Const<N>, U1, ArrayStorage<T, N, 1>>::zeros();

    let mut parts = raw.splitn(N, ',');
    for i in 0..N {
        let element = parts.next().context("Missing vector element")?.trim();
        vec[i] = element
            .parse()
            .context("Can't convert element from string")?;
    }

    Ok(vec)
}
