use super::internal::*;
use super::wgpu_util as gpu;
use std::sync::OnceLock;

pub(crate) static GRAPHICS_CTX: OnceLock<GraphicsCtx> = OnceLock::new();

pub(crate) fn ctx() -> &'static GraphicsCtx {
	GRAPHICS_CTX.get().expect("graphics context not initialized")
}

/// Context: explicit handle to "globals":
///
///   * device and command queue,
///   * shaders
///   * embedded font
///
/// These are, logically speaking, global singletons.
pub(crate) struct GraphicsCtx {
	device: Arc<Device>,
	queue: Queue,

	pub opts: GraphicsOpts,
	pub shader_pack: ShaderPack,
	pub fallback_texture: Arc<Texture>,
}

impl GraphicsCtx {
	pub fn init(opts: &GraphicsOpts, device: Device, queue: Queue, surface_format: TextureFormat) {
		let device = Arc::new(device);

		let fallback_texture = Arc::new(Self::_embedded_fallback_texture((&device, &queue)));
		let shader_pack = ShaderPack::new(device.clone(), &queue, opts, surface_format);

		if GRAPHICS_CTX
			.set(Self {
				device,
				queue,
				opts: opts.clone(),
				shader_pack,
				fallback_texture,
			})
			.is_err()
		{
			panic!("GraphicsCtx::init: already initialized")
		}
	}

	pub fn upload_buffer<T: bytemuck::Pod>(&self, dst: &wgpu::Buffer, src: &[T]) {
		gpu::upload_buffer(self.queue(), dst, src)
	}

	pub fn create_rgba_mipmap(&self, opts: &GraphicsOpts, mips: &[&[u8]], dimensions: uvec2, sampling: &TextureOpts) -> Texture {
		gpu::upload_rgba_mipmap(self.dev_queue(), opts, mips, dimensions, sampling)
	}

	pub fn upload_vao<T>(&self, vertices: &[T], indices: &[u32]) -> VAO
	where
		T: bytemuck::Pod,
	{
		gpu::upload_vao(self.device(), vertices, indices)
	}

	pub fn create_depth_texture(&self, opts: &GraphicsOpts, size: uvec2) -> Texture {
		gpu::create_depth_texture(self.device(), opts, size)
	}

	pub fn create_msaa_fb(&self, opts: &GraphicsOpts, config: &wgpu::SurfaceConfiguration) -> Option<MSAAFB> {
		gpu::maybe_create_msaa_fb(self.device(), opts, config)
	}

	// convenience
	fn dev_queue(&self) -> (&wgpu::Device, &wgpu::Queue) {
		(&self.device, &self.queue)
	}

	pub fn device(&self) -> &wgpu::Device {
		&self.device
	}

	pub fn queue(&self) -> &wgpu::Queue {
		&self.queue
	}

	pub fn upload_image_mip(&self, image: &DynamicImage, sampling: &TextureOpts) -> Texture {
		let mips = gen_mips(&self.opts, image);
		let mips = mips.iter().map(|vec| vec.as_ref()).collect::<Vec<_>>();
		gpu::upload_rgba_mipmap(self.dev_queue(), &self.opts, &mips, image.dimensions().into(), sampling)
	}

	/// Upload RGBA pixel data to the GPU. Linear filtering.
	pub fn upload_image_nomip(&self, image: &DynamicImage, sampling: &TextureOpts) -> Texture {
		let dimensions = image.dimensions();
		let rgba = image.to_rgba8().into_raw();
		self.upload_rgba_mips(&[&rgba], dimensions.into(), sampling)
	}

	/// Upload RGBA pixel data to the GPU. Linear filtering.
	pub fn upload_rgba(&self, rgba: &[u8], dimensions: uvec2, sampling: &TextureOpts) -> Texture {
		self.upload_rgba_mips(&[rgba], dimensions, sampling)
	}

	/// Upload RGBA pixel data to the GPU. Linear filtering.
	pub fn upload_rgba_mips(&self, rgba_mips: &[&[u8]], dimensions: uvec2, sampling: &TextureOpts) -> Texture {
		gpu::upload_rgba_mipmap(self.dev_queue(), &self.opts, rgba_mips, dimensions, sampling)
	}

	fn _embedded_fallback_texture(ctx: (&Device, &Queue)) -> Texture {
		let image = &image::load_from_memory(include_bytes!("fallback_texture.png")).expect("decode embedded texture");
		let opts = GraphicsOpts::default();
		gpu::upload_rgba_mipmap(ctx, &opts, &[&image.to_rgba8()], image.dimensions().into(), &TextureOpts::DEFAULT)
	}

	pub fn upload_meshbuffer(&self, buf: &MeshBuffer) -> VAO {
		self.upload_vao(buf.vertices(), buf.indices())
	}
}

pub(crate) struct MSAAFB {
	pub fb: wgpu::Texture,
	pub fb_view: wgpu::TextureView,
}
