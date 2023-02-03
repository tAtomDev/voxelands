pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_SIZE_I32: i32 = CHUNK_SIZE as i32;
pub const CHUNK_SIZE_LOG2: u32 = 5;
pub const CHUNK_SIZE_CUBED: usize = 32 * 32 * 32;

pub const X_SHIFT: usize = (CHUNK_SIZE_LOG2 + Z_SHIFT as u32) as usize;
pub const Z_SHIFT: usize = CHUNK_SIZE_LOG2 as usize;
pub const Y_SHIFT: usize = 0;

pub const X_MASK: usize = (CHUNK_SIZE - 1) << X_SHIFT;
pub const Z_MASK: usize = (CHUNK_SIZE - 1) << Z_SHIFT;
pub const Y_MASK: usize = CHUNK_SIZE - 1;
