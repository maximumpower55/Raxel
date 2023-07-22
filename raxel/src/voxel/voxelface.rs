use crate::resource::texture_system::TextureId;

#[derive(Clone, Copy, Debug)]
pub struct VoxelFace {
    pub norm: Norm,
    pub tex_id: TextureId,
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Norm {
    NORTH = 0,
    WEST = 1,
    DOWN = 2,

    SOUTH = 3,
    EAST = 4,
    UP = 5,
}

#[allow(dead_code)]
impl Norm {
    pub const VALUES: [Self; 6] = [
        Norm::NORTH,
        Norm::WEST,
        Norm::DOWN,
        Norm::SOUTH,
        Norm::EAST,
        Norm::UP,
    ];
    pub const BITS: u8 = 3;
    pub const BIT_MASK: u8 = (1 << Norm::BITS) - 1;
}
