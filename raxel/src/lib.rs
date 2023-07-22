#![feature(new_uninit)]
#![feature(portable_simd)]
#![feature(array_chunks)]
mod render;
pub mod resource;
pub mod voxel;
pub mod world;

use std::time::Instant;

use futures::executor::block_on;
use render::renderer::{Renderer, RendererState};
use voxel::voxel::VoxelRegistry;
use winit::{
    event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{CursorGrabMode, Window, WindowBuilder},
};
use world::world::World;

static mut VOXEL_REGISTRY: VoxelRegistry = VoxelRegistry::new();

pub struct Instance {
    event_loop: EventLoop<()>,
    window: Window,
    pub renderer: Renderer,

    world: Option<World>,
}

impl Instance {
    pub fn new(
        window_title: Option<&str>,
        register_voxels: &dyn Fn(&'static mut VoxelRegistry),
    ) -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(window_title.unwrap_or("Raxel"))
            .build(&event_loop)
            .unwrap();

        unsafe { register_voxels(&mut VOXEL_REGISTRY) };

        let renderer = block_on(init_renderer(&window));

        Self {
            renderer,
            event_loop,
            window,

            world: None,
        }
    }

    pub fn set_world(&mut self, world: World) {
        self.world = Some(world);
    }

    pub fn run(mut self) {
        let mut frame_start = Instant::now();
        let mut forward = 0.0;
        let mut strafe = 0.0;
        let mut vertical = 0.0;
        self.window.set_cursor_visible(false);
        self.window
            .set_cursor_grab(CursorGrabMode::Confined)
            .unwrap();
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::MainEventsCleared => self.window.request_redraw(),
                Event::WindowEvent { window_id, event } if window_id == self.window.id() => {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::W),
                                ..
                            } => {
                                forward = 1.0;
                            }
                            KeyboardInput {
                                state: ElementState::Released,
                                virtual_keycode: Some(VirtualKeyCode::W),
                                ..
                            } => {
                                forward = 0.0;
                            }
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::S),
                                ..
                            } => {
                                forward = -1.0;
                            }
                            KeyboardInput {
                                state: ElementState::Released,
                                virtual_keycode: Some(VirtualKeyCode::S),
                                ..
                            } => {
                                forward = 0.0;
                            }
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::A),
                                ..
                            } => {
                                strafe = -1.0;
                            }
                            KeyboardInput {
                                state: ElementState::Released,
                                virtual_keycode: Some(VirtualKeyCode::A),
                                ..
                            } => {
                                strafe = 0.0;
                            }
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::D),
                                ..
                            } => {
                                strafe = 1.0;
                            }
                            KeyboardInput {
                                state: ElementState::Released,
                                virtual_keycode: Some(VirtualKeyCode::D),
                                ..
                            } => {
                                strafe = 0.0;
                            }
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Space),
                                ..
                            } => {
                                vertical = 1.0;
                            }
                            KeyboardInput {
                                state: ElementState::Released,
                                virtual_keycode: Some(VirtualKeyCode::Space),
                                ..
                            } => {
                                vertical = 0.0;
                            }
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::LShift),
                                ..
                            } => {
                                vertical = -1.0;
                            }
                            KeyboardInput {
                                state: ElementState::Released,
                                virtual_keycode: Some(VirtualKeyCode::LShift),
                                ..
                            } => {
                                vertical = 0.0;
                            }

                            _ => {}
                        },
                        WindowEvent::Resized(physical_size) => {
                            self.renderer.resize(physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            self.renderer.resize(*new_inner_size);
                        }
                        _ => {}
                    }
                }
                Event::RedrawRequested(_) => {
                    let frame_time = Instant::now().duration_since(frame_start).as_secs_f32();

                    self.renderer.camera.pos.x += (((self.renderer.camera.yaw.cos()) * forward - (self.renderer.camera.yaw.sin()) * strafe)) * frame_time * 180.0;
                    self.renderer.camera.pos.z += (((self.renderer.camera.yaw.sin()) * forward + (self.renderer.camera.yaw.cos()) * strafe)) * frame_time * 180.0;
                    self.renderer.camera.pos.y += vertical * frame_time * 180.0;
                
                    let frame = self.renderer.state.surface.0.get_current_texture().unwrap();
                    self.renderer.render(&frame);
                    frame.present();

                    frame_start = Instant::now();
                }
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    self.renderer.camera.yaw -= (-delta.0 * 0.004) as f32;
                    self.renderer.camera.pitch =
                        (self.renderer.camera.pitch + (-delta.1 * 0.004) as f32).clamp(-1.0, 1.0);
                }
                _ => {}
            }
        });
    }
}

async fn init_renderer(window: &Window) -> Renderer {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    });

    let surface = unsafe { instance.create_surface(&window) }.unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty() | wgpu::Features::MULTI_DRAW_INDIRECT_COUNT | wgpu::Features::POLYGON_MODE_LINE,
                limits: wgpu::Limits {
                    max_buffer_size: 402653184,
                    max_storage_buffer_binding_size: 402653184,
                    max_compute_invocations_per_workgroup: 512,
                    ..Default::default()
                },
            },
            None, // Trace path
        )
        .await
        .unwrap();

    let window_size = window.inner_size();
    let surface_configuration = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8Unorm,
        width: window_size.width,
        height: window_size.height,
        present_mode: wgpu::PresentMode::Immediate,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![wgpu::TextureFormat::Bgra8Unorm],
    };
    surface.configure(&device, &surface_configuration);
    Renderer::new(RendererState {
        surface: (surface, surface_configuration),
        adapter,
        device,
        queue,
    })
}
