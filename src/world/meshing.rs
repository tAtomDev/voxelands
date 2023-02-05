use bevy::{
    math::*,
    render::{mesh::Indices, mesh::Mesh, render_resource::PrimitiveTopology},
};

use crate::{
    data::{voxel_face::*, *},
    rendering::ChunkMaterial,
};

use super::Chunk;

#[derive(Debug, Clone, Default)]
struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub color_intensities: Vec<f32>,
    pub uvs: Vec<[f32; 2]>,
    pub indexes: Vec<u32>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

fn add_face(mesh_data: &mut MeshData, voxel_type: VoxelType, position: IVec3, face: VoxelFace) {
    let indices_offset = mesh_data.positions.len() as u32;
    let face_vertex_positions = face.vertex_positions();
    let face_normal = face.normal().as_vec3();
    let face_index = voxel_type.get_face_index(face);
    let face_color_intensity = face.color_intensity();

    mesh_data.positions.reserve(4);
    mesh_data.normals.reserve(4);
    mesh_data.indexes.reserve(4);
    mesh_data.color_intensities.reserve(4);
    mesh_data.uvs.reserve(8);
    mesh_data.indices.reserve(6);

    for i in 0..6 {
        mesh_data.indices.push(indices_offset + FACE_INDICES[i]);
        if i >= 4 {
            continue;
        }

        let face_position = face_vertex_positions[i] + position.as_vec3();
        mesh_data.positions.push(face_position.into());
        mesh_data.normals.push(face_normal.into());
        mesh_data.indexes.push(face_index);
        mesh_data.color_intensities.push(face_color_intensity);
        mesh_data.uvs.push(FACE_UVS[i].into());
    }
}

pub fn generate_chunk_mesh(chunk: &Chunk) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let mut mesh_data: MeshData = MeshData::default();

    for position in chunk.iter_voxels() {
        let voxel_type = chunk.get_voxel(position);
        if voxel_type.is_transparent() {
            continue;
        }

        for face in FACES {
            if chunk.is_transparent_at(position + face.normal()) {
                add_face(&mut mesh_data, voxel_type, position, face);
            }
        }
    }

    mesh.set_indices(Some(Indices::U32(mesh_data.indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions);
    mesh.insert_attribute(
        ChunkMaterial::ATTRIBUTE_COLOR_INTENSITY,
        mesh_data.color_intensities,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_data.uvs);
    mesh.insert_attribute(ChunkMaterial::ATTRIBUTE_TEXTURE_INDEX, mesh_data.indexes);
    mesh
}

pub fn generate_empty_chunk_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let mesh_data: MeshData = MeshData::default();

    mesh.set_indices(Some(Indices::U32(mesh_data.indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions);
    mesh.insert_attribute(
        ChunkMaterial::ATTRIBUTE_COLOR_INTENSITY,
        mesh_data.color_intensities,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_data.uvs);
    mesh.insert_attribute(ChunkMaterial::ATTRIBUTE_TEXTURE_INDEX, mesh_data.indexes);
    mesh
}
