use super::internal::*;

pub fn save_scene_mesh(map_dir: &MapDir, scene_objects: &Vec<GltfObject>) -> Result<()> {
	println!("saving scene mesh");
	save_bincode_gz(scene_objects, &map_dir.scene_mesh_file())
}

// Suffixes for lightmap file names.
// E.g. `area_sphz` for `deck.map/lm/Cube.001_area_sphz.png`
pub const AREA_SPHZ: &str = "area_sphz";
pub const AREA_SPHX: &str = "area_sphx";
pub const AREA_SPHY: &str = "area_sphy";
pub const DIFFUSE: &str = "diffuse";
pub const OCCUPANCY: &str = "occupancy";
pub const SAMPLES: &str = "samples";
pub const VALIDITY: &str = "validity";
pub const SOURCES: &str = "sources";
pub const SUN_MASK: &str = "sun_mask";
pub const INDIRECT: &str = "indirect";

pub(crate) fn upload_scene_mesh(map_dir: &MapDir, res: &mut Resources, settings: &GraphicsOpts) -> Result<Vec<Object>> {
	let ctx = ctx();
	let mut objects = vec![];

	let scene_objects: Vec<GltfObject> = load_bincode_gz(&map_dir.scene_mesh_file())?;
	let metadata = Metadata::load(map_dir)?;

	let default_normals = uniform_texture(vec4(0.0, 0.0, 1.0, 1.0));

	for scene_obj in scene_objects {
		// load LightAtlas, choose shader based on available channels

		let sphz = load_lightmap(map_dir, &scene_obj, AREA_SPHZ)?;
		let sun_mask = load_lightmap(map_dir, &scene_obj, SUN_MASK)?;

		let sphxy = load_sphxy_maps(map_dir, &scene_obj);

		for Primitive { material, mesh } in &scene_obj.primitives {
			let meshbuf2 = mesh.convert_to_packed()?;
			let vao = Arc::new(ctx.upload_meshbuffer(&meshbuf2));

			let material = metadata.materials.get(material).ok_or_else(|| anyhow!("metadata has no material definition for {material}"))?;

			let diffuse_color = res.textures.load_sync(material.base_color).unwrap_or(res.default_texture.clone());

			let normal_map = match material.normal_map {
				None => None,
				Some(handle) => res.textures.load_sync_with(load_normal_map, handle), // 👈 silently returns None upon failure to load
			};

			// Choose fanciest material possible given the available mappings (normal map, spherical harmonics).
			let material = match (normal_map, &sphxy, settings.shadows) {
				//(None, _) => ctx.shader_pack.lightmap(&diffuse_color, &sphz),// 🪲 lightmap broken does not render sun becuase no normals
				(None, _, _) => ctx.shader_pack.normalmap(&diffuse_color, &sphz, &default_normals, &sun_mask),
				(Some(normal_map), None, _) => ctx.shader_pack.normalmap(&diffuse_color, &sphz, &normal_map, &sun_mask),
				(Some(normal_map), Some((sphx, sphy)), false) => ctx.shader_pack.sph(&diffuse_color, &[&sphx, &sphy, &sphz], &normal_map, &sun_mask),
				(Some(normal_map), Some((sphx, sphy)), true) => ctx.shader_pack.sph_shadows(&diffuse_color, &[&sphx, &sphy, &sphz], &normal_map, &sun_mask),
			};

			objects.push(Object::new(vao, material));
		}
	}

	fn load_lightmap(map_dir: &MapDir, scene_obj: &GltfObject, channel: &str) -> Result<Arc<Texture>> {
		let ctx = GRAPHICS_CTX.get().unwrap();
		//let ctx = ctx(); // can't capture dynamic environment in a fn item WTF
		let file = map_dir.lm_file(channel, scene_obj.name.as_str());
		//LOG.write(format!("loading lightmap {file:?}"));
		log_loading(&file);
		let img = image::open(&file).context(format!("open {file:?}"))?.into_rgb8();
		let img = Arc::new(ctx.upload_image_nomip(&img.clone().into(), ctx.opts.lightmap_filter())); // 👈 TODO: avoid image clone
		Ok(img)
	}

	fn load_normal_map(name: &str) -> Result<Texture> {
		let ctx = GRAPHICS_CTX.get().unwrap();
		let img = load_image(name)?;
		Ok(ctx.upload_image_mip(&img, &RGBA_LINEAR_NORMAL_MAP))
	}

	fn load_sphxy_maps(map_dir: &MapDir, scene_obj: &GltfObject) -> Option<(Texture, Texture)> {
		let sphx = load_sphxy_map(map_dir, scene_obj.name.as_str(), AREA_SPHX).ok()?;
		let sphy = load_sphxy_map(map_dir, scene_obj.name.as_str(), AREA_SPHY).ok()?;
		Some((sphx, sphy))
	}

	fn load_sphxy_map(map_dir: &MapDir, obj_name: &str, infix: &str) -> Result<Texture> {
		let ctx = GRAPHICS_CTX.get().unwrap();
		let file = map_dir.lm_file(infix, obj_name);
		let img = image::open(&file).context(format!("open {file:?}"))?.into_rgb8();
		let img = ctx.upload_image_nomip(&img.clone().into(), &RGBA_LINEAR_NORMAL_MAP);
		Ok(img)
		//Ok(ctx.upload_image_mip(&img, &RGBA_LINEAR_NORMAL_MAP))
	}

	Ok(objects)
}
