use super::bindable::{BindGroupSetter, Bindable};

pub struct BlockBuffer<const CAPACITY: usize, const COUNT: usize> {
    pub blocks: Box<[(usize, usize); COUNT]>,
    buffer: wgpu::Buffer,
    bind_group: Option<wgpu::BindGroup>,
}

impl<const CAPACITY: usize, const COUNT: usize> BlockBuffer<CAPACITY, COUNT> {
    pub fn new(
        device: &wgpu::Device,
        bind_group_layout: Option<wgpu::BindGroupLayout>,
        desc: &wgpu::BufferDescriptor,
    ) -> Self {
        debug_assert!(desc.size == (CAPACITY * COUNT) as u64, "capacity * count has to equal to the size defined in the buffer descriptor!");
        let mut blocks: Box<[(usize, usize); COUNT]> = unsafe { Box::new_zeroed().assume_init() };
        for i in 0..COUNT {
            let last_block: usize = if i == 0 {
                0
            } else {
                blocks.get(i - 1).unwrap_or(&(0, 0)).1
            };
            unsafe { *blocks.get_unchecked_mut(i) = (last_block, CAPACITY * (i + 1)) };
        }
        let buffer = device.create_buffer(desc);
        let bind_group = bind_group_layout.map(|value| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &value,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: None,
            })
        });
        Self {
            blocks,
            buffer,
            bind_group,
        }
    }

    pub fn write_to_block(&self, queue: &wgpu::Queue, block: usize, data: &[u8]) {
        queue.write_buffer(
            &self.buffer,
            unsafe { self.blocks.get_unchecked(block).0 as u64 },
            data,
        )
    }
}

impl<const CAPACITY: usize, const COUNT: usize> Bindable for BlockBuffer<CAPACITY, COUNT> {
    fn bind<'setter>(&'setter self, index: u32, setter: &mut dyn BindGroupSetter<'setter>) {
        unsafe { setter.set_bind_group(index, self.bind_group.as_ref().unwrap_unchecked(), &[]) }
    }

    fn as_entire_binding(&self) -> wgpu::BindingResource {
        self.buffer.as_entire_binding()
    }
}

// pub struct ResizingBlockBuffer {
//     pub blocks: Vec<Range<usize>>,
//     max_size: usize,
//     buffer: wgpu::Buffer,
//     pointer_buffer: (wgpu::Buffer, Vec<u32>),
//     bind_group: Option<wgpu::BindGroup>,
// }

// impl ResizingBlockBuffer {
//     pub fn new(
//         device: &wgpu::Device,
//         bind_group_layout: Option<wgpu::BindGroupLayout>,
//         desc: &wgpu::BufferDescriptor,
//         block_count: usize,
//         block_capacity: usize,
//     ) -> Self {
//         debug_assert!(desc.size == (block_capacity * block_count) as u64, "block_capacity * block_count has to equal to the size defined in the buffer descriptor!");
//         let mut blocks: Vec<Range<usize>> = Vec::with_capacity(block_capacity * block_count);
//         for i in 0..block_count {
//             let last_block: usize = if i == 0 {
//                 0
//             } else {
//                 blocks.get(i - 1).unwrap_or(&(0..0)).end
//             };
//             blocks.push(last_block..block_capacity * (i + 1));
//         }
//         let buffer = device.create_buffer(desc);
//         let pointer_buffer = (
//             device.create_buffer(&wgpu::BufferDescriptor {
//                 label: None,
//                 size: (core::mem::size_of::<u32>() * block_count) as u64,
//                 usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
//                 mapped_at_creation: false,
//             }),
//             vec![0; block_count],
//         );
//         let bind_group = bind_group_layout.map(|value| {
//             device.create_bind_group(&wgpu::BindGroupDescriptor {
//                 layout: &value,
//                 entries: &[
//                     wgpu::BindGroupEntry {
//                         binding: 0,
//                         resource: buffer.as_entire_binding(),
//                     },
//                     wgpu::BindGroupEntry {
//                         binding: 1,
//                         resource: pointer_buffer.0.as_entire_binding(),
//                     },
//                 ],
//                 label: None,
//             })
//         });
//         Self {
//             blocks,
//             max_size: block_capacity * block_count,
//             buffer,
//             pointer_buffer,
//             bind_group,
//         }
//     }

//     pub fn write_to_block(&self, queue: &wgpu::Queue, block: usize, data: &[u8]) {
//         queue.write_buffer(
//             &self.buffer,
//             unsafe { self.blocks.get_unchecked(block).start as u64 },
//             data,
//         )
//     }
// }

// impl Bindable for ResizingBlockBuffer {
//     fn bind<'setter>(&'setter self, index: u32, setter: &mut dyn BindGroupSetter<'setter>) {
//         setter.set_bind_group(index, self.bind_group.as_ref().unwrap(), &[])
//     }

//     fn as_entire_binding(&self) -> wgpu::BindingResource {
//         self.buffer.as_entire_binding()
//     }
// }
