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
    for face_position in face_vertex_positions {
        mesh_data
            .positions
            .push((face_position + position.as_vec3()).into());

        mesh_data.normals.push(face.normal().as_vec3().into());

        mesh_data.indexes.push(voxel_type.get_face_index(face));

        mesh_data.color_intensities.push(face.color_intensity());
    }

    for uv in FACE_UVS {
        mesh_data.uvs.push(uv.into());
    }

    for indice in FACE_INDICES {
        mesh_data.indices.push(indices_offset + indice);
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
