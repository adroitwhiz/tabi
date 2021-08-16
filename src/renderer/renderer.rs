use std::{borrow::Cow, cell::RefCell, collections::HashMap, mem, rc::Rc};

use bytemuck::{Pod, Zeroable};
use futures::executor::block_on;
use glam::Vec2;
use wgpu::util::DeviceExt;
use winit::window::Window;

use super::{
    blank_skin::BlankSkin,
    common::RendererState,
    drawable::{Drawable, DrawableRendererState},
    skin::Skin,
    svg_skin::SVGSkin
};

const NUM_INDICES: usize = 6;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct DrawableID(usize);

pub(super) struct GpuState {
    pub swap_chain: wgpu::SwapChain,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub stage_bind_group: wgpu::BindGroup,
    pub sampler_nearest: wgpu::Sampler,
    pub sampler_linear: wgpu::Sampler,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct StageUniforms {
    size: [f32; 2],
}

pub struct Renderer {
    gpu_state: GpuState,
    drawable_renderer_state: DrawableRendererState,
    drawables: HashMap<DrawableID, Drawable>,
    draw_list: Vec<DrawableID>,
    skins: Vec<Rc<RefCell<dyn Skin>>>,
    stage_size: (u32, u32),
    next_drawable_id: usize
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    _pos: [f32; 2],
    _tex_coord: [f32; 2],
}

static QUAD_VERTS: [Vertex; 4] = [
    Vertex {
        _pos: [-0.5, -0.5],
        _tex_coord: [0.0, 1.0],
    },
    Vertex {
        _pos: [0.5, -0.5],
        _tex_coord: [1.0, 1.0],
    },
    Vertex {
        _pos: [-0.5, 0.5],
        _tex_coord: [0.0, 0.0],
    },
    Vertex {
        _pos: [0.5, 0.5],
        _tex_coord: [1.0, 0.0],
    },
];

static QUAD_INDICES: [u16; NUM_INDICES] = [0, 1, 2, 1, 2, 3];

impl Renderer {
    pub fn with_window(window: &Window, size: (u32, u32), stage_size: (u32, u32)) -> Self {
        let instance = wgpu::Instance::new(wgpu::BackendBit::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        }))
        .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        ))
        .expect("Failed to create device");

        let drawable_renderer_state = DrawableRendererState::init(&device);

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&QUAD_VERTS),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&QUAD_INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });

        // Load the shaders from disk
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
            flags: wgpu::ShaderFlags::all(),
        });

        let stage_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Drawable"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(8),
                    },
                    count: None,
                }],
            });

        let stage_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&StageUniforms {
                size: [stage_size.0 as f32, stage_size.1 as f32],
            }),
            usage: wgpu::BufferUsage::UNIFORM,
        });

        let stage_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &stage_bind_group_layout,
            label: Some("Stage"),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: stage_uniform_buf.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &stage_bind_group_layout,
                &drawable_renderer_state.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: (mem::size_of::<f32>() * 2) as u64,
                    shader_location: 1,
                },
            ],
        }];

        let swapchain_format = wgpu::TextureFormat::Bgra8Unorm;

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: swapchain_format.into(),
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrite::default()
                }],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
        });

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.0,
            height: size.1,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let sampler_nearest= device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("mip"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let sampler_linear= device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("mip"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let gpu_state = GpuState {
            swap_chain,
            sc_desc,
            surface,
            device,
            queue,
            render_pipeline,
            vertex_buf,
            index_buf,
            stage_bind_group,
            sampler_nearest,
            sampler_linear
        };

        Self {
            gpu_state,
            drawable_renderer_state,
            drawables: HashMap::new(),
            draw_list: Vec::new(),
            skins: Vec::new(),
            stage_size,
            next_drawable_id: 0
        }
    }

    fn draw_these(&mut self, texture_view: &wgpu::TextureView) {
        let mut encoder = self
            .gpu_state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let drawables = &mut self.drawables;
        for drawable_id in self.draw_list.iter() {
            let drawable = drawables
                .get_mut(drawable_id)
                .expect("Drawable does not exist--did the draw list get out of sync with the set of drawables?");
            drawable.update_bind_group(&self.gpu_state);
        }

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.gpu_state.render_pipeline);
            rpass.set_index_buffer(
                self.gpu_state.index_buf.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            rpass.set_vertex_buffer(0, self.gpu_state.vertex_buf.slice(..));
            rpass.set_bind_group(0, &self.gpu_state.stage_bind_group, &[]);

            for drawable_id in self.draw_list.iter() {
                let drawable = drawables.get(drawable_id).unwrap();
                rpass.set_bind_group(1, &drawable.bind_group, &[]);
                rpass.draw_indexed(0..NUM_INDICES as u32, 0, 0..1);
            }
        }

        let command_buffers = Some(encoder.finish());

        self.gpu_state.queue.submit(command_buffers);
    }

    pub fn draw(&mut self) {
        let frame = self
            .gpu_state
            .swap_chain
            .get_current_frame()
            .expect("Failed to acquire next swap chain texture")
            .output;

        self.draw_these(&frame.view);
    }

    pub fn resize(&mut self, size: (u32, u32)) {
        self.gpu_state.sc_desc.width = size.0;
        self.gpu_state.sc_desc.height = size.1;
        self.gpu_state.swap_chain = self
            .gpu_state
            .device
            .create_swap_chain(&self.gpu_state.surface, &self.gpu_state.sc_desc);
    }

    pub fn create_blank_skin(&mut self) -> Rc<RefCell<dyn Skin>> {
        let s = Rc::new(RefCell::new(BlankSkin::new(Vec2::new(100f32, 75f32), Vec2::new(50.0, 37.5))));
        self.skins.push(s);
        Rc::clone(&self.skins[self.skins.len() - 1])
    }

    pub fn create_svg_skin(&mut self, svg_data: &[u8], rotation_center: (f64, f64)) -> Rc<RefCell<dyn Skin>> {
        let s = Rc::new(RefCell::new(SVGSkin::new(&self.gpu_state, svg_data, Vec2::new(rotation_center.0 as f32, rotation_center.0 as f32))));
        self.skins.push(s);
        Rc::clone(&self.skins[self.skins.len() - 1])
    }

    fn add_to_draw_list(&mut self, drawable_id: DrawableID) {
        self.draw_list.push(drawable_id);
    }

    pub fn create_drawable(&mut self, skin: Rc<RefCell<dyn Skin>>) -> DrawableID {
        let next_drawable_id = self.next_drawable_id;
        self.next_drawable_id += 1;
        let id = DrawableID(next_drawable_id);
        let d = Drawable::new(
            skin,
            &self.gpu_state,
            &self.drawable_renderer_state,
        );
        self.drawables.insert(id, d);
        self.add_to_draw_list(id);
        id
    }

    pub fn update_drawable_position(&mut self, drawable_id: DrawableID, position: (f64, f64)) {
        self.drawables.get_mut(&drawable_id).expect("Invalid drawable ID").set_position(Vec2::new(position.0 as f32, position.1 as f32))
    }

    pub fn update_drawable_rotation_scale(&mut self, drawable_id: DrawableID, rotation: f32, scale: (f64, f64)) {
        let drawable = self.drawables.get_mut(&drawable_id).expect("Invalid drawable ID");
        drawable.set_rotation(rotation);
        drawable.set_scale(Vec2::new(scale.0 as f32, scale.1 as f32));
    }
}
