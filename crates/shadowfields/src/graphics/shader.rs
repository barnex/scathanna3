use super::internal::*;

// Shaders are not intended to be manually constructed.
// Use `ShaderPack` instead.
#[derive(Clone)]
pub(crate) enum Shader {
	Flat(Arc<wgpu::BindGroup>),
	Lines(Arc<wgpu::BindGroup>),
	Lightmap(Arc<wgpu::BindGroup>),
	Normalmap(Arc<wgpu::BindGroup>),
	Sph(Arc<wgpu::BindGroup>),
	Text(Arc<wgpu::BindGroup>),
	Highlight(Arc<wgpu::BindGroup>),
	Entity(Arc<wgpu::BindGroup>, InstanceData),
	Particles(Arc<wgpu::BindGroup>, InstanceData),
	Debris(Arc<wgpu::BindGroup>, InstanceData),
	Animation(Arc<wgpu::BindGroup>, InstanceData),
}
