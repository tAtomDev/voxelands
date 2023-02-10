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
    pub indices: Vec<u32>,
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub color_intensities: Vec<f32>,
    pub indexes: Vec<u32>,
}

#[inline(always)]
fn add_face(mesh_data: &mut MeshData, voxel_type: VoxelType, position: IVec3, face: VoxelFace) {
    let indices_offset = mesh_data.positions.len() as u32;
    let face_vertex_positions = face.vertex_positions();
    let face_index = voxel_type.get_face_index(face);
    let face_color_intensity = face.color_intensity();

    let mut indices: Vec<u32> = vec![0; 6];
    let mut positions: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0]; 4];
    let mut uvs: Vec<[f32; 2]> = vec![[0.0, 0.0]; 4];
    let mut color_intensities: Vec<f32> = vec![0.0; 4];
    let mut indexes: Vec<u32> = vec![0; 4];

    for i in 0..6 {
        indices[i] = indices_offset + FACE_INDICES[i];
        if i >= 4 {
            continue;
        }

        let face_position = face_vertex_positions[i] + position.as_vec3();
        positions[i] = face_position.into();
        indexes[i] = face_index;
        color_intensities[i] = face_color_intensity;
        uvs[i] = FACE_UVS[i].into();
    }

    mesh_data.indices.extend_from_slice(&indices);
    mesh_data.positions.extend_from_slice(&positions);
    mesh_data.uvs.extend_from_slice(&uvs);
    mesh_data
        .color_intensities
        .extend_from_slice(&color_intensities);
    mesh_data.indexes.extend_from_slice(&indexes);
}

#[inline(always)]
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

    //for face in FACES {
    //    add_face(&mut mesh_data, VoxelType::Stone, (0, 0, 0).into(), face);
    //}

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

#[test]
fn t() {
    let mut chunk = Chunk::new((0, 0, 0).into());
    for position in chunk.iter_voxels() {
        chunk.set_voxel(VoxelType::Dirt, position);
    }
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let mut mesh_data: MeshData = MeshData::default();

    for position in chunk.iter_voxels() {
        let voxel_type = chunk.get_voxel(position);
        //if voxel_type.is_transparent() {
        //    continue;
        //}

        for face in FACES {
            //if chunk.is_transparent_at(position + face.normal()) {
            add_face(&mut mesh_data, voxel_type, position, face);
            //}
        }
    }

    println!("{}", mesh_data.positions.len());
    println!("{}", mesh_data.indices.len());
    println!("{}", mesh_data.color_intensities.len());
    println!("{}", mesh_data.uvs.len());
    println!("{}", mesh_data.indexes.len());

    mesh.set_indices(Some(Indices::U32(mesh_data.indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions);
    mesh.insert_attribute(
        ChunkMaterial::ATTRIBUTE_COLOR_INTENSITY,
        mesh_data.color_intensities,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_data.uvs);
    mesh.insert_attribute(ChunkMaterial::ATTRIBUTE_TEXTURE_INDEX, mesh_data.indexes);
}
