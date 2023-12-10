///! Low-level WGPU utilities with performance counters.
use super::internal::counters::Counters;
use super::internal::*;
use std::mem;

/// Performance counters for all WGPU calls going through this layer.
pub(crate) static COUNTERS: Counters = Counters::new();

/// Upload an arbitrary buffer (e.g. vertex positions, uniform buffers,...) to the GPU.
pub(crate) fn upload_buffer<T: bytemuck::Pod>(queue: &Queue, dst: &wgpu::Buffer, src: &[T]) {
	COUNTERS.buffer_uploads.inc();
	COUNTERS.bytes_uploaded.add((src.len() * mem::size_of::<T>()) as u64);
	queue.write_buffer(dst, 0, bytemuck::cast_slice(src));
}

/// Create a `VAO` (vertices + indices) on the GPU.
pub(crate) fn upload_vao<T>(device: &Device, vertices: &[T], indices: &[u32]) -> VAO
where
	T: bytemuck::Pod,
{
	debug_assert!(indices.len() < 1 << 31);
	COUNTERS.buffer_creates.inc();
	let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
		label: Some(file!()),
		contents: bytemuck::cast_slice(vertices),
		usage: wgpu::BufferUsages::VERTEX,
	});
	let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
		label: Some(file!()),
		contents: bytemuck::cast_slice(indices),
		usage: wgpu::BufferUsages::INDEX,
	});

	VAO {
		num_indices: indices.len() as u32,
		vertex_buffer,
		index_buffer,
	}
}

/// Upload mipmap data in RGBA8 format to the GPU.
/// `mips` has the image data from the largest mipmap level to the lowest mipmap level.
/// `dimensions` is the size of the largest level, must be a power of two if there's more than 1 level.
pub(crate) fn upload_rgba_mipmap((device, queue): (&Device, &Queue), opts: &GraphicsOpts, mips: &[&[u8]], dimensions: uvec2, sampling: &TextureOpts) -> Texture {
	COUNTERS.texture_uploads.inc();

	debug_assert!(mips[0].len() == 4 * dimensions.reduce(u32::mul) as usize);
	if mips.len() > 1 {
		assert!(dimensions.x().is_power_of_two());
		assert!(dimensions.y().is_power_of_two());
	}

	let mut size = wgpu::Extent3d {
		width: dimensions.x(),
		height: dimensions.y(),
		depth_or_array_layers: 1,
	};
	let texture = device.create_texture(&wgpu::TextureDescriptor {
		label: Some(file!()),
		size,
		mip_level_count: mips.len() as u32,
		sample_count: 1,
		dimension: wgpu::TextureDimension::D2,
		format: sampling.format,
		view_formats: &[sampling.format],
		usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
	});

	for (i, rgba) in mips.iter().enumerate() {
		let i = i as u32;
		queue.write_texture(
			wgpu::ImageCopyTexture {
				aspect: wgpu::TextureAspect::All,
				texture: &texture,
				mip_level: i,
				origin: wgpu::Origin3d::ZERO,
			},
			rgba,
			wgpu::ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(4 * size.width),
				rows_per_image: Some(size.height),
			},
			size,
		);
		size.width /= 2;
		size.height /= 2;
	}

	let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

	// When anisotropic clamp is not 1, all filter modes must be linear
	let anisotropy_clamp = match sampling.max_filter {
		FilterMode::Linear => opts.anisotropy_clamp(),
		_ => 1,
	};

	let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
		address_mode_u: sampling.address_mode,
		address_mode_v: sampling.address_mode,
		address_mode_w: sampling.address_mode,
		mag_filter: sampling.max_filter,
		min_filter: sampling.max_filter,
		mipmap_filter: sampling.max_filter,
		label: Some(file!()),
		anisotropy_clamp,
		..default()
	});

	Texture { texture, view, sampler }
}

/// Create a depth texture suitable for the multisampling (MSAA) settings in `opts`.
pub(crate) fn create_depth_texture(device: &Device, opts: &GraphicsOpts, size: uvec2) -> Texture {
	let size = wgpu::Extent3d {
		width: size.x(),
		height: size.y(),
		depth_or_array_layers: 1,
	};
	let desc = wgpu::TextureDescriptor {
		label: Some(file!()),
		size,
		mip_level_count: 1,
		sample_count: opts.msaa_sample_count(),
		dimension: wgpu::TextureDimension::D2,
		format: Canvas::DEPTH_FORMAT,
		view_formats: &[Canvas::DEPTH_FORMAT],
		usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
	};
	let texture = device.create_texture(&desc);
	let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
	let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
		address_mode_u: wgpu::AddressMode::ClampToEdge,
		address_mode_v: wgpu::AddressMode::ClampToEdge,
		address_mode_w: wgpu::AddressMode::ClampToEdge,
		mag_filter: wgpu::FilterMode::Linear,
		min_filter: wgpu::FilterMode::Linear,
		mipmap_filter: wgpu::FilterMode::Nearest,
		compare: Some(wgpu::CompareFunction::LessEqual),
		lod_min_clamp: 0.0,
		lod_max_clamp: 100.0,
		..default()
	});

	Texture { texture, view, sampler }
}

/// Create a multi-sampled framebuffer, if MSAA is enabled in the graphics settings.
pub(crate) fn maybe_create_msaa_fb(device: &Device, opts: &GraphicsOpts, config: &wgpu::SurfaceConfiguration) -> Option<MSAAFB> {
	if opts.msaa {
		let fb = create_multisampled_framebuffer(device, opts.msaa_sample_count(), config);
		let fb_view = fb.create_view(&wgpu::TextureViewDescriptor::default());
		Some(MSAAFB { fb, fb_view })
	} else {
		None
	}
}

fn create_multisampled_framebuffer(device: &Device, sample_count: u32, config: &wgpu::SurfaceConfiguration) -> wgpu::Texture {
	let multisampled_texture_extent = wgpu::Extent3d {
		width: config.width,
		height: config.height,
		depth_or_array_layers: 1,
	};
	let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
		size: multisampled_texture_extent,
		mip_level_count: 1,
		sample_count,
		dimension: wgpu::TextureDimension::D2,
		format: config.format,
		view_formats: &[config.format],
		usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
		label: Some(file!()),
	};
	device.create_texture(multisampled_frame_descriptor)
}
