pub(super) trait RendererState {
    fn init(device: &wgpu::Device) -> Self;
}
