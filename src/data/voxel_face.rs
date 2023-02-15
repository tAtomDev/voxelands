use bevy::math::*;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VoxelFace {
    Left,
    Right,
    Bottom,
    Top,
    Back,
    Front,
}

impl VoxelFace {
    pub const fn is_side_face(&self) -> bool {
        matches!(
            self,
            VoxelFace::Left | VoxelFace::Right | VoxelFace::Front | VoxelFace::Back
        )
    }
}

pub const FACES: [VoxelFace; 6] = [
    VoxelFace::Left,
    VoxelFace::Right,
    VoxelFace::Bottom,
    VoxelFace::Top,
    VoxelFace::Back,
    VoxelFace::Front,
];

pub const FACE_VERTEX_POSITIONS: [[Vec3; 4]; 6] = [
    [
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(-0.5, 0.5, 0.5),
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(-0.5, -0.5, 0.5),
    ],
    [
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, -0.5),
    ],
    [
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(0.5, -0.5, -0.5),
    ],
    [
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(-0.5, 0.5, 0.5),
        Vec3::new(0.5, 0.5, 0.5),
    ],
    [
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(-0.5, -0.5, -0.5),
    ],
    [
        Vec3::new(-0.5, 0.5, 0.5),
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, 0.5),
    ],
];

pub const FACE_NORMALS: [IVec3; 6] = [
    IVec3::new(-1, 0, 0),
    IVec3::new(1, 0, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, 0, -1),
    IVec3::new(0, 0, 1),
];

pub const FACE_COLOR_INTENSITIES: [u32; 6] = [3, 3, 2, 5, 4, 4];

#[rustfmt::skip]
impl VoxelFace {
    pub const fn normal(&self) -> IVec3 {
        FACE_NORMALS[*self as usize]
    }

    pub const fn color_intensity(&self) -> u32 {
        FACE_COLOR_INTENSITIES[*self as usize]
    }

    pub const fn vertex_positions(&self) -> [Vec3; 4] {
        FACE_VERTEX_POSITIONS[*self as usize]
    }
}

pub const FACE_UVS: [Vec2; 4] = [
    Vec2::new(0.0, 0.0),
    Vec2::new(1.0, 0.0),
    Vec2::new(0.0, 1.0),
    Vec2::new(1.0, 1.0),
];

pub const FACE_INDICES: [u32; 6] = [2, 1, 0, 2, 3, 1];
