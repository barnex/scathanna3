use super::internal::*;

pub(super) struct GlobalsBuffer {
	pub buffer: wgpu::Buffer, // Stores GlobalsHostData
	pub bind_group: wgpu::BindGroup,
}

/// Data buffer to be uploaded as global uniform data (shaders: `struct Globals`).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub(crate) struct GlobalsHostData {
	// global camera
	view_proj: [[f32; 4]; 4],

	cam_position: vec3,
	_padding0: f32, // ☠️ wrong padding can silently break shader ☠️

	// global sun_direction
	sun_dir: vec3,
	_padding1: f32, // ☠️

	sun_color: vec3,
	_padding2: f32, // ☠️ be still, my wgpu.
}

impl GlobalsHostData {
	pub fn from(camera: &Camera, viewport_size: uvec2, sun_dir: vec3, sun_color: vec3) -> Self {
		debug_assert!(approx_eq(sun_dir.len(), 1.0));
		Self {
			view_proj: camera.matrix(viewport_size),
			cam_position: camera.position,
			sun_dir,
			sun_color,
			_padding0: default(),
			_padding1: default(),
			_padding2: default(),
		}
	}
}

impl GlobalsBuffer {
	pub fn new(device: &wgpu::Device) -> Self {
		let hostdata = GlobalsHostData::default();
		let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some(file!()),
			contents: bytemuck::cast_slice(&[hostdata]),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		});

		let bind_group_layout = Self::bind_group_layout(device);

		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &bind_group_layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: buffer.as_entire_binding(),
			}],
			label: Some(file!()),
		});
		Self { buffer, bind_group }
	}

	pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			entries: &[wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
				/// <<<<<<<<<<<<<<<  ????????????? just VERTEX?
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			}],
			label: Some(file!()),
		})
	}
}
