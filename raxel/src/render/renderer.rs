use std::simd::{ SimdPartialEq, simd_swizzle, usizex4, Which::*, u32x16 };

use ultraviolet::Mat4;
use winit::dpi::PhysicalSize;

use crate::{
    resource::{
        resource::{LoadedResource, ResourceId, ResourceLoader},
        texture_system::{create_texture_array, TextureId},
    },
    world::{
        cell::{
            Cell, CELL_BUFFER_SIZE, CELL_SIZE, CELL_Z_MASK, CELL_X_MASK, CELL_Y_MASK,
        },
        world::{World, WORLD_LENGTH, WORLD_SIZE},
    },
    VOXEL_REGISTRY,
};

use super::{
    bindable::{Bindable, BindableBuffer},
    block_buffer::BlockBuffer,
    camera::Camera,
};

pub struct RendererState {
    pub surface: (wgpu::Surface, wgpu::SurfaceConfiguration),
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

pub struct Renderer {
    pub state: RendererState,
    pub camera: Camera,
    count_buffer: wgpu::Buffer,
    indirect_buffer: wgpu::Buffer,
    vertex_count_buffer: BlockBuffer<4, WORLD_SIZE>,
    command_gen_bind_group: wgpu::BindGroup,
    face_buffer: BlockBuffer<{ CELL_BUFFER_SIZE as usize }, WORLD_SIZE>,
    camera_buffer: BindableBuffer,
    tex_bind_group: wgpu::BindGroup,
    cell_pipeline: wgpu::RenderPipeline,
    command_gen_pipeline: wgpu::ComputePipeline,
    depth_texture: wgpu::TextureView,
}

struct MeshingRun {
    pub tex: TextureId,
    pub width: u8,
    pub height: u8,
}

impl Renderer {
    pub fn new(state: RendererState) -> Self {
        let face_bind_group_layout =
            state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                CELL_BUFFER_SIZE * WORLD_SIZE as u64,
                            ),
                        },
                        count: None,
                    }],
                });

        let texture_bind_group_layout =
            state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2Array,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let camera_bind_group_layout =
            state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(128),
                        },
                        count: None,
                    }],
                });

        let cell_pipeline_layout =
            state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[
                        &face_bind_group_layout,
                        &camera_bind_group_layout,
                        &texture_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let LoadedResource::SHADER(cell_shader_source) = ResourceId(Some(String::from("shader")), String::from("cell.wgsl")).load(ResourceLoader::SHADER) else { unreachable!() };
        let cell_pipeline = state
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&cell_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &state
                        .device
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: None,
                            source: cell_shader_source.clone(),
                        }),
                    entry_point: "vert",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &state
                        .device
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: None,
                            source: cell_shader_source,
                        }),
                    entry_point: "frag",
                    targets: &[Some(state.surface.1.view_formats[0].into())],
                }),
                primitive: wgpu::PrimitiveState {
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

        let LoadedResource::SHADER(command_gen_shader_source) = ResourceId(Some(String::from("shader")), String::from("command_gen.wgsl")).load(ResourceLoader::SHADER) else { unreachable!() };
        let command_gen_pipeline =
            state
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: None,
                    layout: None,
                    module: &state
                        .device
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: None,
                            source: command_gen_shader_source,
                        }),
                    entry_point: "main",
                });

        let depth_texture = Self::create_depth_texture(&state);

        let count_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: core::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT,
            mapped_at_creation: false,
        });

        let indirect_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (core::mem::size_of::<wgpu::util::DrawIndirect>() * WORLD_SIZE) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT,
            mapped_at_creation: false,
        });

        let vertex_count_buffer = BlockBuffer::new(
            &state.device,
            None,
            &wgpu::BufferDescriptor {
                label: None,
                size: (core::mem::size_of::<u32>() * WORLD_SIZE) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );

        let command_gen_bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &command_gen_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: count_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: indirect_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: vertex_count_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });

        let face_buffer = BlockBuffer::new(
            &state.device,
            Some(face_bind_group_layout),
            &wgpu::BufferDescriptor {
                label: None,
                size: CELL_BUFFER_SIZE * WORLD_SIZE as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );

        let block_tex_array_view = create_texture_array(&state.device, &state.queue, 16, 16)
            .create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..Default::default()
            });

        let block_tex_sampler = state.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let tex_bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&block_tex_array_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&block_tex_sampler),
                },
            ],
            label: None,
        });

        let camera = Camera::new(state.surface.1.width as f32 / state.surface.1.height as f32);
        let camera_buffer = BindableBuffer::new(
            &state.device,
            &camera_bind_group_layout,
            state.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: core::mem::size_of::<Mat4>() as u64 * 2,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );

        Self {
            state,
            camera,
            count_buffer,
            indirect_buffer,
            vertex_count_buffer,
            command_gen_bind_group,
            face_buffer,
            camera_buffer,
            tex_bind_group,
            cell_pipeline,
            command_gen_pipeline,
            depth_texture,
        }
    }

    fn create_depth_texture(state: &RendererState) -> wgpu::TextureView {
        state
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: state.surface.1.width,
                    height: state.surface.1.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn mesh_cell(&mut self, cell: &Cell, world: &World) {
        let mut faces: [[u32; CELL_SIZE]; 6] = [[0; CELL_SIZE]; 6];

        for z in 0..32 { for y in 0..32 { for x in 0..32 {
            let idx = Cell::encode(x, y, z);
            let voxel_id = unsafe { *cell.voxels.get_unchecked(idx) };
            let model = unsafe { VOXEL_REGISTRY.get(voxel_id).model };
            if model.is_none() {
                continue;
            }

            const NEIGHBOR_MASKS: usizex4 = usizex4::from_array([CELL_Z_MASK, CELL_X_MASK, CELL_Y_MASK, 0]);
            const NEIGHBOR_DECODING_MASKS: usizex4 = usizex4::from_array([31, 31, 31, 0]);
            const NEIGHBOR_DECODING: usizex4 = usizex4::from_array([0, 10, 5, 0]);
            let idx_splat = usizex4::splat(idx);
            let invalid_negative_neighbors = ((idx_splat >> NEIGHBOR_DECODING) & NEIGHBOR_DECODING_MASKS).simd_eq(usizex4::splat(0));
            let invalid_positive_neighbors = ((idx_splat >> NEIGHBOR_DECODING) & NEIGHBOR_DECODING_MASKS).simd_eq(usizex4::splat(31));

            let invalid_neighbors = simd_swizzle!(invalid_negative_neighbors.to_int(), invalid_positive_neighbors.to_int(), [
                First(0),
                First(1),
                First(2),
                Second(0),
                Second(1),
                Second(2),
                Second(3),
                Second(3),
            ]);

            let neighbor_indices = simd_swizzle!(idx_splat - NEIGHBOR_MASKS, idx_splat + NEIGHBOR_MASKS, [
                First(0),
                First(1),
                First(2),
                Second(0),
                Second(1),
                Second(2),
                Second(3),
                Second(3),
            ]);

            for i in 0..6usize {
                let face = unsafe { *model.unwrap().0.get_unchecked(i) };
                let neighbor_index = neighbor_indices[i];

                if invalid_neighbors[i] == 0 {
                    if unsafe { VOXEL_REGISTRY.get(*cell.voxels.get_unchecked(neighbor_index)) }.model.is_some() {
                        continue;
                    }
                }

                unsafe { *faces.get_unchecked_mut(i).get_unchecked_mut(idx) = (0 << 0) | (0 << 5) | ((face.tex_id as u32) << 10)};
            }
        }}}

        let mut mesh: Vec<u32> = Vec::new();
        for i in 0..6usize {
            let faces = unsafe { faces.get_unchecked(i) };

            const CHUNK_SIZE: usize = 16;
            let mut chunk_index = 0;
            for chunk in faces.array_chunks::<CHUNK_SIZE>() {
                let arr = u32x16::from_array(*chunk);
                let mut chunk_mesh = u32x16::from_array([
                    chunk_index + 0,
                    chunk_index + 1,
                    chunk_index + 2,
                    chunk_index + 3,
                    chunk_index + 4,
                    chunk_index + 5,
                    chunk_index + 6,
                    chunk_index + 7,
                    chunk_index + 8,
                    chunk_index + 9,
                    chunk_index + 10,
                    chunk_index + 11,
                    chunk_index + 12,
                    chunk_index + 13,
                    chunk_index + 14,
                    chunk_index + 15,
                ]);

                chunk_mesh |= u32x16::splat((i as u32) << 15);
                chunk_mesh |= ((arr >> u32x16::splat(10)) & u32x16::splat(7)) << u32x16::splat(18);
                chunk_mesh |= ((arr >> u32x16::splat(0)) & u32x16::splat(31)) << u32x16::splat(21);
                chunk_mesh |= ((arr >> u32x16::splat(5)) & u32x16::splat(31)) << u32x16::splat(26);

                for j in 0..16usize {
                    if chunk[j] != 0 { mesh.push(chunk_mesh[j]); }
                }

                chunk_index += CHUNK_SIZE as u32;
            }
        }

        let idx = World::encode(cell.pos.x, cell.pos.y, cell.pos.z);
        self.face_buffer.write_to_block(
            &self.state.queue,
            idx,
            bytemuck::cast_slice(&mesh.as_slice()),
        );
        self.vertex_count_buffer.write_to_block(
            &self.state.queue,
            idx,
            &(6 * mesh.len() as u32).to_le_bytes(),
        );
    }

    pub fn render(&self, frame: &wgpu::SurfaceTexture) {
        self.state.queue.write_buffer(
            &self.camera_buffer.buffer,
            0,
            bytemuck::cast_slice(&[self.camera.matrices()]),
        );

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());

            pass.set_pipeline(&self.command_gen_pipeline);
            pass.set_bind_group(0, &self.command_gen_bind_group, &[]);
            pass.dispatch_workgroups(WORLD_LENGTH as u32, WORLD_LENGTH as u32, WORLD_LENGTH as u32);
        }

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            pass.set_pipeline(&self.cell_pipeline);
            self.camera_buffer.bind(1, &mut pass);
            self.face_buffer.bind(0, &mut pass);
            pass.set_bind_group(2, &self.tex_bind_group, &[]);
            pass.multi_draw_indirect_count(
                &self.indirect_buffer,
                0,
                &self.count_buffer,
                0,
                WORLD_SIZE as u32,
            );
        }

        self.state.queue.submit(Some(encoder.finish()));
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 { return; }

        let mut surface_configuration = &mut self.state.surface.1;

        surface_configuration.width = new_size.width;
        surface_configuration.height = new_size.height;
        self.camera.aspect =
            surface_configuration.width as f32 / surface_configuration.height as f32;

        self.state
            .surface
            .0
            .configure(&self.state.device, &surface_configuration);

        self.depth_texture = Self::create_depth_texture(&self.state);
    }
}
