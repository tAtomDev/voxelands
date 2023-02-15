use bevy::{
    math::*,
    render::{mesh::Indices, mesh::Mesh, render_resource::PrimitiveTopology},
};

use crate::{
    data::{constants::CHUNK_SIZE, voxel_face::*, *},
    rendering::ChunkMaterial,
};

use super::Chunk;

#[derive(Debug, Clone, Default)]
struct MeshData {
    pub indices: Vec<u32>,
    pub positions: Vec<[f32; 3]>,
    pub data: Vec<u32>,
}

impl MeshData {
    pub fn new() -> MeshData {
        MeshData {
            indices: Vec::new(),
            positions: Vec::new(),
            data: Vec::new(),
        }
    }
}

#[inline(always)]
fn add_face(mesh_data: &mut MeshData, voxel_type: VoxelType, position: IVec3, face: VoxelFace) {
    let indices_offset = mesh_data.positions.len() as u32;
    let face_vertex_positions = face.vertex_positions();
    let face_index = voxel_type.get_face_index(face);
    let face_color_intensity = face.color_intensity();

    for i in 0..6 {
        mesh_data.indices.push(indices_offset + FACE_INDICES[i]);

        if i >= 4 {
            continue;
        }

        let face_position = face_vertex_positions[i] + position.as_vec3();
        mesh_data.positions.push(face_position.into());

        let uvx = FACE_UVS[i].x as u32;
        let uvy = FACE_UVS[i].y as u32;

        let data: u32 = (face_index << 6) | (face_color_intensity << 2) | (uvx << 1) | uvy;
        mesh_data.data.push(data);
    }
}

#[inline(always)]
pub fn generate_chunk_mesh(chunk: &Chunk) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let mut mesh_data: MeshData = MeshData::new();

    //for position in chunk.iter_voxels() {
    //    let voxel_type = chunk.get_voxel(position);
    //    if voxel_type.is_transparent() {
    //        continue;
    //    }
    //
    //    for face in FACES {
    //        if chunk.is_transparent_at(position + face.normal()) {
    //            add_face(&mut mesh_data, voxel_type, position, face);
    //        }
    //    }
    //}

    mesh.set_indices(Some(Indices::U32(mesh_data.indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions);
    mesh.insert_attribute(ChunkMaterial::ATTRIBUTE_DATA, mesh_data.data);
    mesh
}

pub fn generate_empty_chunk_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let mesh_data: MeshData = MeshData::default();

    mesh.set_indices(Some(Indices::U32(mesh_data.indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions);
    mesh.insert_attribute(ChunkMaterial::ATTRIBUTE_DATA, mesh_data.data);
    mesh
}

#[test]
fn t() {
    let mut chunk = Chunk::new((0, 0, 0).into());
    for position in chunk.iter_voxels() {
        chunk.set_voxel(VoxelType::Dirt, position);
    }
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let mut mesh_data: MeshData = MeshData::new();

    for position in chunk.iter_voxels() {
        let voxel_type = chunk.get_voxel(position);

        for face in FACES {
            add_face(&mut mesh_data, voxel_type, position, face);
        }
    }

    println!("{}", mesh_data.positions.len());
    println!("{}", mesh_data.indices.len());
    println!("{}", mesh_data.data.len());

    mesh.set_indices(Some(Indices::U32(mesh_data.indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions);
    mesh.insert_attribute(ChunkMaterial::ATTRIBUTE_DATA, mesh_data.data);
}
