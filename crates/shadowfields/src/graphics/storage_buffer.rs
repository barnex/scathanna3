use super::internal::*;

pub(crate) struct StorageBuffer<T> {
	pub host_data: Vec<T>,
	pub buffer: wgpu::Buffer,
	pub bindings: wgpu::BindGroup,
	capacity: usize,
}

impl<T> StorageBuffer<T>
where
	T: bytemuck::Pod + Default,
{
	pub fn with_capacity(device: &wgpu::Device, capacity: usize) -> Self {
		let host_data = Vec::with_capacity(capacity);
		let mut data = vec![T::default(); capacity];
		let buffer = Self::new_buffer(device, &data);
		let bindings = Self::make_bind_group(device, &buffer);
		data.clear();
		Self {
			host_data,
			buffer,
			bindings,
			capacity,
		}
	}

	pub fn upload(&self, ctx: &GraphicsCtx) {
		let n = usize::min(self.capacity, self.host_data.len());
		ctx.upload_buffer(&self.buffer, &self.host_data[..n]);
	}

	fn new_buffer(device: &wgpu::Device, data: &[T]) -> wgpu::Buffer {
		device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some(file!()),
			contents: bytemuck::cast_slice(data),
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
		})
	}

	fn bind_group_layout_desc() -> wgpu::BindGroupLayoutDescriptor<'static> {
		wgpu::BindGroupLayoutDescriptor {
			label: Some(file!()),
			entries: &[wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::VERTEX,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Storage { read_only: true },
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			}],
		}
	}

	pub fn make_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
		device.create_bind_group_layout(&Self::bind_group_layout_desc())
	}

	pub fn make_bind_group(device: &wgpu::Device, buffer: &wgpu::Buffer) -> wgpu::BindGroup {
		let bind_group_layout = device.create_bind_group_layout(&Self::bind_group_layout_desc());
		device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some(file!()),
			layout: &bind_group_layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: buffer.as_entire_binding(),
			}],
		})
	}
}
