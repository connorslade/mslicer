//! This module contains the definition for the [`mesh::Mesh`] struct along with methods for the actual slicing operations.

use nalgebra::Vector3;

pub mod builder;
pub mod format;
pub mod half_edge;
pub mod mesh;
pub mod segments;
pub mod slicer;
pub mod supports;

pub type Pos = Vector3<f32>;
