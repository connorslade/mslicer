use std::{
    fs::File,
    io::{BufReader, BufWriter},
    mem,
    sync::Arc,
};

use anyhow::Result;

use common::{
    progress::Progress,
    serde::{ReaderDeserializer, WriterSerializer},
};
use mesh_format::Format;
use slicer::{
    half_edge::HalfEdgeMesh,
    mesh::{Mesh, MeshInner},
};
use tools::repair::{MeshRepair, RepairResult};

fn main() -> Result<()> {
    println!("[*] Loading");
    let file = File::open("/home/connorslade/Documents/Resin Printing/Test Models/Skull_v1.stl")?;
    let des = ReaderDeserializer::new(BufReader::new(file));
    let mesh = mesh_format::load_mesh(des, Format::Stl, &Progress::new())?;

    let mesh = Mesh::new_boxed(mesh.verts, mesh.faces);

    println!("[*] Generating Half Edge Mesh");
    let half_edge = Arc::new(HalfEdgeMesh::build(mesh.inner()));

    println!("[*] Repairing");
    let RepairResult {
        mesh,
        unwelded_vertices,
    } = MeshRepair {
        vertex_epsilon: 1e-4,
    }
    .repair(&mesh, half_edge);

    println!("{{ unwelded_vertices: {unwelded_vertices} }}");

    println!("[*] Saving");
    let repair =
        unsafe { mem::transmute::<Arc<MeshInner>, Arc<mesh_format::Mesh>>(mesh.inner().clone()) };

    let file = File::create("repair.stl")?;
    let mut ser = WriterSerializer::new(BufWriter::new(file));
    mesh_format::save_mesh(&mut ser, Format::Stl, &Progress::new(), &repair);

    Ok(())
}
