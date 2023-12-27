use super::internal::*;

// User settings for graphics quality.
// TODO: flags vs. opts (no_msaa vs. msaa)
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct GraphicsOpts {
	pub width: u32,
	pub height: u32,
	pub fullscreen: bool,
	pub anisotropy: u8,
	pub texture_resolution: u32,
	pub normal_maps: bool,
	pub msaa: bool,
	pub textures: bool,
	pub mipmaps: bool,
	pub trilinear: bool,
	pub lightmap_nearest: bool,
	pub vsync: bool,
	pub shadows: bool,
	#[serde(default = "default_true")]
	pub hud: bool,
}

fn default_true() -> bool {
	true
}

impl GraphicsOpts {
	pub fn msaa_sample_count(&self) -> u32 {
		// currently WGPU only supports 1 or 4 samples (https://github.com/gfx-rs/wgpu/issues/1832)
		match self.msaa {
			true => 4,
			false => 1,
		}
	}

	pub fn anisotropy_clamp(&self) -> u16 {
		// must be at least 1.
		match self.anisotropy {
			0 | 1 => 1,
			2 | 4 | 8 | 16 => self.anisotropy as u16,
			_ => {
				log::error!("invalid anisotropy: {}", self.anisotropy);
				1
			} // invalid. TODO: check on start-up
		}
	}

	pub fn lightmap_filter(&self) -> &TextureOpts {
		match self.lightmap_nearest {
			true => &NEAREST,
			false => &TextureOpts::DEFAULT,
		}
	}
}

impl Default for GraphicsOpts {
	fn default() -> Self {
		Self {
			width: 1280,
			height: 768,
			fullscreen: false,
			msaa: false,
			anisotropy: 16,
			texture_resolution: 512,
			normal_maps: true,
			textures: true,
			mipmaps: false,
			trilinear: false,
			lightmap_nearest: false,
			vsync: true,
			shadows: true,
			hud: true,
		}
	}
}
