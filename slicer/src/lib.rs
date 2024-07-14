//! This module contains the definition for the [`mesh::Mesh`] struct along with methods for the actual slicing operations.

use nalgebra::Vector3;

pub mod mesh;
pub mod segments;
pub mod slicer;

pub type Pos = Vector3<f32>;
