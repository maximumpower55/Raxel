use ultraviolet::UVec3;

use crate::render::renderer::Renderer;

use super::cell::Cell;

pub const WORLD_LENGTH: usize = 8;
pub const WORLD_SIZE: usize = (WORLD_LENGTH * WORLD_LENGTH * WORLD_LENGTH) as usize;

#[derive(Debug)]
pub struct World {
    pub cells: Box<[Cell; WORLD_SIZE]>,
}

impl World {
    pub fn new(renderer: &mut Renderer, cell_generator: &dyn Fn(&mut Cell)) -> Self {
        let mut cells: Box<[Cell; WORLD_SIZE]> = unsafe { Box::new_zeroed().assume_init() };
        for idx in 0..WORLD_SIZE {
            let x = idx >> 6 & WORLD_LENGTH - 1;
            let y = idx >> 3 & WORLD_LENGTH - 1;
            let z = idx >> 0 & WORLD_LENGTH - 1;

            let cell = unsafe { cells.get_unchecked_mut(idx) };
            cell.pos = UVec3::new(x as u32, y as u32, z as u32);
            cell_generator(cell);
        }

        let world = Self {
            cells,
        };
        for idx in 0..WORLD_SIZE {
            renderer.mesh_cell(unsafe { world.cells.get_unchecked(idx) }, &world);
        }
        world
    }

    #[inline(always)]
    pub const fn encode(x: u32, y: u32, z: u32) -> usize {
        ((x as usize) << 6) | ((y as usize) << 3) | ((z as usize) << 0)
    }
}
