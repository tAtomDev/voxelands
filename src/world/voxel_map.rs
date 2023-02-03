use bevy::prelude::IVec3;

use crate::data::{constants::*, VoxelType};

#[inline]
const fn flatten(position: IVec3) -> usize {
    (position.x as usize) << X_SHIFT
        | (position.y as usize) << Y_SHIFT
        | (position.z as usize) << Z_SHIFT
}

#[inline]
const fn unflatten(index: usize) -> IVec3 {
    IVec3::new(
        ((index & X_MASK) >> X_SHIFT) as i32,
        ((index & Y_MASK) >> Y_SHIFT) as i32,
        ((index & Z_MASK) >> Z_SHIFT) as i32,
    )
}

#[derive(Default)]
pub struct VoxelIterator {
    index: usize,
}

impl Iterator for VoxelIterator {
    type Item = IVec3;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= CHUNK_SIZE_CUBED {
            None
        } else {
            let xyz = unflatten(self.index);
            self.index += 1;
            Some(xyz)
        }
    }
}

#[derive(Debug, Clone)]
pub struct VoxelMap {
    data: [VoxelType; CHUNK_SIZE_CUBED],
}

impl VoxelMap {
    pub const fn new() -> Self {
        Self {
            data: [VoxelType::Air; CHUNK_SIZE_CUBED],
        }
    }

    #[inline(always)]
    pub const fn is_within_bounds(local: IVec3) -> bool {
        local.x >= 0
            && local.x < CHUNK_SIZE_I32
            && local.z >= 0
            && local.z < CHUNK_SIZE_I32
            && local.y >= 0
            && local.y < CHUNK_SIZE_I32
    }

    pub const fn iter(&self) -> VoxelIterator {
        VoxelIterator { index: 0 }
    }

    pub const fn get_at(&self, position: IVec3) -> VoxelType {
        if !VoxelMap::is_within_bounds(position) {
            VoxelType::Air
        } else {
            self.data[flatten(position)]
        }
    }

    pub fn set_at(&mut self, voxel: VoxelType, position: IVec3) {
        if VoxelMap::is_within_bounds(position) {
            self.data[flatten(position)] = voxel;
        }
    }
}

#[test]
fn test() {
    println!("{}", flatten((31, 0, 0).into()));
    println!("{}", unflatten(18446744073709550592));
}
