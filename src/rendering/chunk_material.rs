use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexAttribute,
        render_resource::{AsBindGroup, ShaderRef, VertexFormat},
    },
};

#[derive(Resource, Clone, Default)]
pub struct ChunkTextureAtlas(pub Handle<Image>);

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct ChunkMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub texture_atlas: Handle<Image>,
}

impl ChunkMaterial {
    pub const ATTRIBUTE_COLOR_INTENSITY: MeshVertexAttribute =
        MeshVertexAttribute::new("Color_Intensity", 2 << 12, VertexFormat::Float32);

    pub const ATTRIBUTE_TEXTURE_INDEX: MeshVertexAttribute =
        MeshVertexAttribute::new("Texture_Index", 2 << 13, VertexFormat::Uint32);
}

impl Material for ChunkMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/chunk.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/chunk.wgsl".into()
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::render::mesh::MeshVertexBufferLayout,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
            ChunkMaterial::ATTRIBUTE_COLOR_INTENSITY.at_shader_location(2),
            ChunkMaterial::ATTRIBUTE_TEXTURE_INDEX.at_shader_location(3),
        ])?;

        descriptor.vertex.buffers = vec![vertex_layout];

        Ok(())
    }
}
