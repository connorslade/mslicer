use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

pub mod vector3f {
    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vector3<f32>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let [x, y, z] = <[f32; 3]>::deserialize(deserializer)?;
        Ok(Vector3::new(x, y, z))
    }

    pub fn serialize<S>(data: &Vector3<f32>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        data.as_slice().serialize(serializer)
    }
}

pub mod vector2u {
    use nalgebra::Vector2;

    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vector2<u32>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let [x, y] = <[u32; 2]>::deserialize(deserializer)?;
        Ok(Vector2::new(x, y))
    }

    pub fn serialize<S>(data: &Vector2<u32>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        data.as_slice().serialize(serializer)
    }
}

pub mod vector3_list {
    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Vector3<f32>>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = Vec::<f32>::deserialize(deserializer)?;
        Ok(data
            .chunks(3)
            .map(|chunk| Vector3::new(chunk[0], chunk[1], chunk[2]))
            .collect())
    }

    pub fn serialize<S>(data: &[Vector3<f32>], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let out = data.iter().flat_map(|v| v.iter()).collect::<Vec<_>>();
        out.serialize(serializer)
    }
}

pub mod index_list {
    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<[u32; 3]>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = Vec::<u32>::deserialize(deserializer)?;
        Ok(data
            .chunks(3)
            .map(|chunk| [chunk[0], chunk[1], chunk[2]])
            .collect())
    }

    pub fn serialize<S>(data: &[[u32; 3]], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let out = data.iter().flat_map(|v| v.iter()).collect::<Vec<_>>();
        out.serialize(serializer)
    }
}
