use super::voxel_face::VoxelFace;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VoxelType {
    Air,
    Grass,
    Dirt,
    //    Stone,
}

impl VoxelType {
    pub fn is_transparent(&self) -> bool {
        *self == VoxelType::Air
    }

    pub const fn get_face_index(&self, face: VoxelFace) -> u32 {
        match self {
            VoxelType::Grass if face.is_side_face() => 1,
            VoxelType::Grass => match face {
                VoxelFace::Top => 0,
                _ => 2,
            },
            VoxelType::Dirt => 2,
            //            VoxelType::Stone => 3,
            _ => 0,
        }
    }
}
