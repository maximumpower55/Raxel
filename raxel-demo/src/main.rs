use raxel::{
    resource::{
        resource::{ResourceId, ResourceLoader},
        texture_system::add_texture,
    },
    voxel::{voxel::Voxel, voxelmodel::VoxelModel},
    world::world::{World, WORLD_LENGTH},
};
use simdnoise::NoiseBuilder;

static mut AIR: usize = 0;
static mut DIRT: usize = 0;
static mut STONE: usize = 0;
static mut GRASS: usize = 0;

pub fn main() {
    let mut instance = raxel::Instance::new(None, &|voxel_registry| unsafe {
        let _ = add_texture(
            ResourceId(Some("texture".to_string()), "dirt.png".to_string())
                .load(ResourceLoader::TEXTURE),
        );
        let _ = add_texture(
            ResourceId(Some("texture".to_string()), "stone.png".to_string())
                .load(ResourceLoader::TEXTURE),
        );
        let _ = add_texture(
            ResourceId(Some("texture".to_string()), "grass_top.png".to_string())
                .load(ResourceLoader::TEXTURE),
        );
        let _ = add_texture(
            ResourceId(Some("texture".to_string()), "grass_side.png".to_string())
                .load(ResourceLoader::TEXTURE),
        );

        AIR = voxel_registry.register(Voxel { model: None });
        DIRT = voxel_registry.register(Voxel {
            model: Some(VoxelModel::all(0)),
        });
        STONE = voxel_registry.register(Voxel {
            model: Some(VoxelModel::all(1)),
        });
        GRASS = voxel_registry.register(Voxel {
            model: Some(VoxelModel::top_bottom(0, 2, 3)),
        });
    });
    let noise = NoiseBuilder::gradient_2d(32 * WORLD_LENGTH as usize, 32 * WORLD_LENGTH as usize)
        .with_freq(0.01)
        .generate_scaled(0.0, 1.0);
    let world = World::new(&mut instance.renderer, &|cell| unsafe {
        for x in 0..32 {
            for z in 0..32 {
                let height = ((((*noise.get_unchecked(
                    ((x + cell.pos.x * 32) << 8 | (z + cell.pos.z * 32) << 0) as usize,
                ) + 1.0)
                    / 230.0)
                    * 5000.0)
                    .round() as u32
                    + 30
                ).min(224);

                for y in 0..32 {
                    let world_y = y + cell.pos.y * 32;
                    if world_y > height {
                        continue;
                    } else if world_y == height {
                        cell.set(x as u8, y as u8, z as u8, GRASS);
                    } else if world_y > height - 5 {
                        cell.set(x as u8, y as u8, z as u8, DIRT);
                    } else {
                        cell.set(x as u8, y as u8, z as u8, STONE);
                    }
                }
            }
        }
    });
    instance.set_world(world);
    instance.run();
}
