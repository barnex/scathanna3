use super::internal::*;

#[derive(Clone)]
pub(crate) struct GMaterial {
	pub diffuse: Arc<Texture>,
	pub normal: Option<Arc<Texture>>,
}

impl GMaterial {
	/// Load a material from diffuse/normal/... maps under, e.g., assets/materials/256/material_name/material_name_normal.png.
	/// (the file layout by <http://www.sharetextures.com>)
	pub fn upload(host: &HostMaterial) -> Self {
		let ctx = ctx();
		Self {
			diffuse: Arc::new(ctx.upload_image_mip(&host.diffuse.clone(/*TODO: avoid clone because DynamicImage*/).into(), &default())),
			normal: host.normal_vec.as_ref().map(|img| Arc::new(Self::upload_normal_map(img))),
		}
	}

	fn upload_normal_map(normal_map: &Img<vec3>) -> Texture {
		let ctx = ctx();
		ctx.upload_image_mip(&encode_normal_map(normal_map).into(), &RGBA_LINEAR_NORMAL_MAP)
	}

	pub fn fallback() -> Self {
		let ctx = ctx();
		Self {
			diffuse: ctx.fallback_texture.clone(),
			normal: None,
		}
	}

	pub fn uniform(color: vec3) -> Self {
		Self {
			diffuse: Arc::new(uniform_texture(color.append(1.0))),
			normal: None,
		}
	}
}
