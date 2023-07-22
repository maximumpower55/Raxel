use crate::resource::texture_system::TextureId;

use super::voxelface::{Norm, VoxelFace};

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct VoxelModel(pub [VoxelFace; 6]);

impl VoxelModel {
    pub fn all(tex_id: TextureId) -> Self {
        Self {
            0: [
                VoxelFace {
                    norm: Norm::NORTH,
                    tex_id,
                },
                VoxelFace {
                    norm: Norm::WEST,
                    tex_id,
                },
                VoxelFace {
                    norm: Norm::DOWN,
                    tex_id,
                },
                VoxelFace {
                    norm: Norm::SOUTH,
                    tex_id,
                },
                VoxelFace {
                    norm: Norm::EAST,
                    tex_id,
                },
                VoxelFace {
                    norm: Norm::UP,
                    tex_id,
                },
            ],
        }
    }

    pub fn top_bottom(
        bottom_tex_id: TextureId,
        top_tex_id: TextureId,
        side_tex_id: TextureId,
    ) -> Self {
        Self {
            0: [
                VoxelFace {
                    norm: Norm::NORTH,
                    tex_id: side_tex_id,
                },
                VoxelFace {
                    norm: Norm::WEST,
                    tex_id: side_tex_id,
                },
                VoxelFace {
                    norm: Norm::DOWN,
                    tex_id: bottom_tex_id,
                },
                VoxelFace {
                    norm: Norm::SOUTH,
                    tex_id: side_tex_id,
                },
                VoxelFace {
                    norm: Norm::EAST,
                    tex_id: side_tex_id,
                },
                VoxelFace {
                    norm: Norm::UP,
                    tex_id: top_tex_id,
                },
            ],
        }
    }
}
