use super::internal::*;

#[derive(Copy, Clone)]
pub struct TextureOpts {
	pub max_filter: wgpu::FilterMode,
	pub format: wgpu::TextureFormat,
	pub address_mode: wgpu::AddressMode,
}

pub const NEAREST: TextureOpts = TextureOpts {
	max_filter: wgpu::FilterMode::Nearest,
	..TextureOpts::DEFAULT
};

// RGB in Linear color space (NOT sRGB!). Only suitable for normal maps.
pub const RGBA_LINEAR_NORMAL_MAP: TextureOpts = TextureOpts {
	format: wgpu::TextureFormat::Rgba8Unorm,
	..TextureOpts::DEFAULT
};

pub const CLAMP_TO_EDGE: TextureOpts = TextureOpts {
	address_mode: wgpu::AddressMode::ClampToEdge,
	..TextureOpts::DEFAULT
};

//pub const RGBA_LINEAR: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

impl TextureOpts {
	pub const DEFAULT: Self = Self {
		max_filter: wgpu::FilterMode::Linear,
		format: wgpu::TextureFormat::Rgba8UnormSrgb,
		address_mode: wgpu::AddressMode::Repeat,
	};

	pub fn filter(self, filter: wgpu::FilterMode) -> Self {
		self.with(|s| s.max_filter = filter)
	}
	pub fn format(self, format: wgpu::TextureFormat) -> Self {
		self.with(|s| s.format = format)
	}
	pub fn address_mode(self, mode: wgpu::AddressMode) -> Self {
		self.with(|s| s.address_mode = mode)
	}
}

impl Default for TextureOpts {
	fn default() -> Self {
		Self::DEFAULT
	}
}
