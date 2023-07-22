pub trait BindGroupSetter<'a> {
    fn set_bind_group(
        &mut self,
        index: u32,
        bind_group: &'a wgpu::BindGroup,
        offsets: &[wgpu::DynamicOffset],
    );
}

impl<'a> BindGroupSetter<'a> for wgpu::RenderPass<'a> {
    fn set_bind_group(
        &mut self,
        index: u32,
        bind_group: &'a wgpu::BindGroup,
        offsets: &[wgpu::DynamicOffset],
    ) {
        self.set_bind_group(index, bind_group, offsets)
    }
}

impl<'a> BindGroupSetter<'a> for wgpu::ComputePass<'a> {
    fn set_bind_group(
        &mut self,
        index: u32,
        bind_group: &'a wgpu::BindGroup,
        offsets: &[wgpu::DynamicOffset],
    ) {
        self.set_bind_group(index, bind_group, offsets)
    }
}

pub trait Bindable {
    fn bind<'setter>(&'setter self, index: u32, setter: &mut dyn BindGroupSetter<'setter>);
    fn as_entire_binding(&self) -> wgpu::BindingResource;
}

pub struct BindableBuffer {
    pub buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl BindableBuffer {
    pub fn new(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        buffer: wgpu::Buffer,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
        });
        Self { buffer, bind_group }
    }
}

impl Bindable for BindableBuffer {
    fn bind<'setter>(&'setter self, index: u32, setter: &mut dyn BindGroupSetter<'setter>) {
        setter.set_bind_group(index, &self.bind_group, &[])
    }

    fn as_entire_binding(&self) -> wgpu::BindingResource {
        self.buffer.as_entire_binding()
    }
}
