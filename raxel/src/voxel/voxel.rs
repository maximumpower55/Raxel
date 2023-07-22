use super::voxelmodel::VoxelModel;

#[derive(Clone, Copy, Debug)]
pub struct Voxel {
    pub model: Option<VoxelModel>,
}

#[derive(Clone, Debug)]
pub struct VoxelRegistry {
    last_id: usize,
    lookup: Vec<Voxel>,
}

impl VoxelRegistry {
    pub const fn new() -> Self {
        VoxelRegistry {
            last_id: 0,
            lookup: Vec::new(),
        }
    }

    pub fn register(&mut self, voxel: Voxel) -> usize {
        self.lookup.insert(self.last_id, voxel);
        self.last_id += 1;

        self.last_id - 1
    }

    #[inline(always)]
    pub fn get(&self, id: usize) -> &Voxel {
        unsafe { self.lookup.get_unchecked(id) }
    }
}
