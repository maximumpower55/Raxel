use ultraviolet::UVec3;

pub const CELL_LENGTH: usize = 32;
pub const CELL_SIZE: usize = CELL_LENGTH * CELL_LENGTH * CELL_LENGTH;
pub const CELL_BUFFER_SIZE: u64 = (core::mem::size_of::<u32>() * 6 * CELL_SIZE) as u64;

pub const CELL_X_MASK: usize = Cell::encode(1, 0, 0);
pub const CELL_Y_MASK: usize = Cell::encode(0, 1, 0);
pub const CELL_Z_MASK: usize = Cell::encode(0, 0, 1);

#[derive(Debug)]
pub struct Cell {
    pub voxels: [usize; CELL_SIZE],
    pub pos: UVec3,
}

impl Cell {
    pub fn new(pos: UVec3) -> Self {
        Self {
            voxels: [0; CELL_SIZE],
            pos: pos,
        }
    }

    #[inline(always)]
    pub const fn encode(x: u8, y: u8, z: u8) -> usize {
        ((x as usize) << 10) | ((y as usize) << 5) | ((z as usize) << 0)
    }

    pub fn set(&mut self, x: u8, y: u8, z: u8, id: usize) {
        unsafe {
            *self
                .voxels
                .get_unchecked_mut(Self::encode(x, y, z) as usize) = id;
        }
    }

    pub fn get(&self, x: u8, y: u8, z: u8) -> usize {
        unsafe { *self.voxels.get_unchecked(Self::encode(x, y, z) as usize) }
    }
}
