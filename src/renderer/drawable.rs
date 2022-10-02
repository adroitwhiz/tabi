use std::cell::RefCell;
use std::rc::Rc;

use bytemuck::{Pod, Zeroable};
use glam::{Affine2, Vec2};

use super::common::RendererState;
use super::renderer::GpuState;
use super::skin::Skin;

pub(super) struct DrawableRendererState {
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl RendererState for DrawableRendererState {
    fn init(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Drawable"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(24),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        DrawableRendererState { bind_group_layout }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct DrawableUniforms {
    matrix: [f32; 6],
}

pub struct Drawable {
    matrix: Affine2,
    skin: Rc<RefCell<dyn Skin>>,
    position: Vec2,
    rotation: f32,
    scale: Vec2,
    matrix_dirty: bool,
    inverse_dirty: bool,
    bind_group_dirty: bool,

    uniform_buf: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Drawable {
    pub(super) fn new(
        skin: Rc<RefCell<dyn Skin>>,
        gpu_state: &GpuState,
        state: &DrawableRendererState,
    ) -> Self {
        let uniform_buf = gpu_state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Drawable.uniform_buf"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: 24,
            mapped_at_creation: false,
        });

        let bind_group = gpu_state
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &state.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            skin.borrow_mut().get_texture(1f32),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&gpu_state.sampler_linear),
                    },
                ],
                label: None,
            });

        Self {
            matrix: Affine2::default(),
            skin,
            position: Vec2::default(),
            rotation: 0f32,
            scale: Vec2::new(1f32, 1f32),
            matrix_dirty: true,
            inverse_dirty: true,
            bind_group_dirty: true,

            uniform_buf,
            bind_group,
        }
    }

    fn set_matrix_dirty(&mut self) {
        self.matrix_dirty = true;
        self.inverse_dirty = true;
        self.bind_group_dirty = true;
    }

    // TODO
    fn calculate_transform(&mut self) {
        let skin_size = self.skin.borrow_mut().get_size();
        self.matrix = Affine2::from_scale_angle_translation(
            skin_size * self.scale,
            self.rotation,
            self.position,
        );
        //self.matrix = Mat3::default();
        self.matrix_dirty = false;
    }

    pub fn get_position(&self) -> Vec2 {
        self.position
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.position = position.round();
        self.set_matrix_dirty();
    }

    pub fn get_rotation(&self) -> f32 {
        self.rotation
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
        self.set_matrix_dirty();
    }

    pub fn get_scale(&self) -> Vec2 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: Vec2) {
        self.scale = scale;
        self.set_matrix_dirty();
    }

    pub fn get_matrix(&mut self) -> Affine2 {
        if self.matrix_dirty {
            self.calculate_transform();
        }
        self.matrix
    }

    pub fn get_skin(&self) -> Rc<RefCell<dyn Skin>> {
        Rc::clone(&self.skin)
    }

    pub fn set_skin(&mut self, skin: Rc<RefCell<dyn Skin>>) {
        self.skin = skin;
        self.set_matrix_dirty();
    }

    pub(super) fn update_bind_group(&mut self, gpu_state: &GpuState) {
        if self.bind_group_dirty {
            let mat = self.get_matrix();
            gpu_state.queue.write_buffer(
                &self.uniform_buf,
                0,
                bytemuck::bytes_of(&DrawableUniforms {
                    matrix: mat.to_cols_array(),
                }),
            );
            self.bind_group_dirty = false;
        }
    }
}
