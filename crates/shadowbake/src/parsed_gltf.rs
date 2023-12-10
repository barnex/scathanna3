use crate::prelude::*;
use gltf::khr_lights_punctual::Light;

/// See `parse_gltf`.
#[derive(Default, Clone)]
pub struct ParsedGltf {
	/// Meshes that share a lightmap.
	pub objects: Vec<GltfObject>,

	pub metadata: Metadata,
}

/// Interpret and validate a GLTF (+BIN) file for use as game map.
///
/// "Interpret" means:
///   * Flatten the node hierarchy and propagate transforms from parent to child
///   * Parse blender custom properties (GLTF "extras"), and modify affected nodes accordingly
///     (E.g. turn a cube into a spawn point. See `CustomProperties` for their meaning).
///
/// The `ParsedGltf` will still need to be baked and serialized. See `convert_gltf`.
pub(crate) fn parse_gltf(map_dir: &MapDir) -> Result<ParsedGltf> {
	let (gltf, buffers) = load_map_gltf(map_dir)?;

	let mut parsed = ParsedGltf::default();
	parsed.metadata.materials = parse_materials(&gltf)?;

	// recursive add to `parsed`.
	let scene = get_single_scene(&gltf)?;
	for node in scene.nodes() {
		parse_node(&buffers, &mut parsed, &mat4::UNIT, &node, 1).with_context(|| format!("Node {}", node.name().unwrap_or_default()))?;
	}
	Ok(parsed)
}

// Use this material name for nodes without material.
// We add this default material to the parsed material palette as well (mapping to plain gray).
const DEFAULT_MATERIAL: &str = "default";

fn parse_materials(gltf: &gltf::Document) -> Result<MaterialPalette> {
	let mut materials = MaterialPalette::default();
	for material in gltf.materials() {
		let (material_handle, material_def) = parse_gltf_material(&material)?;
		if let Some(prev) = materials.insert(material_handle, material_def.clone()) {
			if prev != material_def {
				bail!("multiple conflicting material definitions for {:?}:\n {:#?}\n\nversus:\n\n {:#?}", material.name(), prev, material_def)
			}
		}
	}

	// Handle missing blender materials gracefully.
	if !materials.contains_key(&handle(DEFAULT_MATERIAL)) {
		materials.insert(handle(DEFAULT_MATERIAL), MaterialDef::default());
	}

	Ok(materials)
}

/// Recursively parse a GLTF node and its children. Add results to `parsed`.
fn parse_node(buffers: &Buffers, parsed: &mut ParsedGltf, parent_transform: &mat4, node: &gltf::Node, depth: u32) -> Result<()> {
	println!("{}↳🔷 node: {}", padding(depth), node.name().unwrap_or("<Unnamed>"),);

	let name = node.name().ok_or_else(|| anyhow!("Node does not have a name"))?.to_string();

	// chain transforms
	let transform = parent_transform * &mat4::from(node.transform().matrix());

	parse_light(parsed, &transform, node, depth + 1)?;
	let custom_properties = CustomProperties::parse(node.extras())?;
	if let Some(mesh) = node.mesh() {
		// Only check transform if there's actually a mesh.
		// Nodes like Armature Bones don't have a mesh and are effectively ignored for game purposes.
		// So no need to check their transform.
		check_transform(node.transform().decomposed())?;
		let meshes = parse_mesh(&buffers, &transform, mesh, depth + 1)?; // incl material

		apply_custom_properties(parsed, node, &meshes, &custom_properties, depth + 1)?;

		// only add mesh if not hidden by custom properties (e.g. spawn points are not rendered)
		if !custom_properties.should_hide_mesh() && meshes.len() != 0 {
			parsed.objects.push(GltfObject {
				name: Handle::from_str(&name)?,
				primitives: meshes,
			})
		} else {
			println!("{}↳❌ mesh hidden", padding(depth + 2));
		}
	}

	// recurse
	for child in node.children() {
		parse_node(buffers, parsed, &transform, &child, depth + 1)?;
	}

	Ok(())
}

/// Error out if the transform is unsupported.
/// (anisotropic scale complicates normals. Use "Object > Apply > All Transforms" in Blender to work around this.).
fn check_transform((_translation, _rotation, scale): ([f32; 3], [f32; 4], [f32; 3])) -> Result<()> {
	if !(approx_eq(scale[1], scale[0]) && approx_eq(scale[2], scale[0])) {
		bail!("anisotropic scale not supported: {scale:?}")
	}
	Ok(())
}

// Turn directional light into sun (there can only be one).
// Convert point sources.
fn parse_light(parsed_gltf: &mut ParsedGltf, transform: &mat4, node: &gltf::Node, depth: u32) -> Result<()> {
	if let Some(light) = node.light() {
		match light.kind() {
			gltf::khr_lights_punctual::Kind::Directional => parse_sun(parsed_gltf, transform, &light, depth + 1)?,
			gltf::khr_lights_punctual::Kind::Point => parse_point_light(parsed_gltf, transform, &light, depth + 1)?,
			gltf::khr_lights_punctual::Kind::Spot { .. } => bail!("Unhandled light type: Spot"),
		};
	}
	Ok(())
}

fn parse_sun(parsed_gltf: &mut ParsedGltf, transform: &mat4, light: &Light, depth: u32) -> Result<()> {
	if parsed_gltf.metadata.sun_def.is_some() {
		bail!("Cannot have more than one directional light (sun)");
	}
	let color = vec3::from(light.color()); // Note: ignoring light.intensity(): blender uses W/m2, we use just color
	let dir = (transform * vec4(0.0, 0.0, -1.0, 0.0)).xyz().safe_normalized();
	parsed_gltf.metadata.sun_def = Some(SunDef { dir, color });

	println!("{}↳☀️ sun: {}: {:?}", padding(depth), light.name().unwrap_or_default(), parsed_gltf.metadata.sun_def,);

	Ok(())
}

fn parse_point_light(parsed_gltf: &mut ParsedGltf, transform: &mat4, light: &Light, depth: u32) -> Result<()> {
	// ❓ Blender intensity units are unclear.
	// Experimentally, A 1000W lamp appears to saturate SRGB color space (value 255) at 1m distance (test19_intensity.map).
	// 1000W gets exported to GLTF as an intensity of 54351.4 (test19_intensity.map/map.gltf)
	// (presumably in candela as per https://github.com/KhronosGroup/glTF/tree/main/extensions/2.0/Khronos/KHR_lights_punctual#light-types)
	// We use this as our intensity unit, so that saturated light in Blender.
	// corresponds to saturated light in our units.
	// 🪲 Blender's fall-off appears to be different though, possibly due to tone mapping.
	//const UNIT_INTENSITY: f32 = 54351.4;
	const UNIT_INTENSITY: f32 = 200.0;

	let light_def = PointLightDef {
		pos: (&transform).mul(vec4(0.0, 0.0, 0.0, 1.0)).xyz(), // 👈 position is determined solely by transform.
		color: (light.intensity() / UNIT_INTENSITY) * vec3::from(light.color()),
		range: light.range().unwrap_or(INF), // 👈 TODO: calculate cutoff intensity
	};

	println!(
		"{}↳ 💡 point light: {}: color: {:?}, intensity: {}, range: {:?}",
		padding(depth),
		light.name().unwrap_or_default(),
		light.color(),
		light.intensity(),
		light.range(),
	);
	println!("{}=>💡 point_light_def: pos: {}, color: {}, range: {}", padding(depth), light_def.pos, light_def.color, light_def.range,);

	parsed_gltf.metadata.point_lights.push(light_def);
	Ok(()) // 🪲
}

fn parse_mesh(buffers: &Buffers, transform: &mat4, mesh: gltf::Mesh, depth: u32) -> Result<Vec<Primitive>> {
	let mut result = vec![];

	println!("{}↳🔻 mesh {}", padding(depth), mesh.name().unwrap_or_default());
	for primitive in mesh.primitives() {
		// convert material
		let material = Handle::from_str(primitive.material().name().unwrap_or(DEFAULT_MATERIAL))?;
		println!("{}↳◒ material: {}", padding(depth + 1), primitive.material().name().unwrap_or_default());

		if primitive.mode() != gltf::mesh::Mode::Triangles {
			bail!("unsupported triangle mode {:?}", primitive.mode())
		}

		// convert trimesh
		let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
		let indices = reader.read_indices().ok_or_else(|| anyhow!("no indices"))?.into_u32().collect_vec();

		let positions = reader
			.read_positions()
			.ok_or_else(|| anyhow!("no positions"))?
			.map(vec3::from)
			.map(|position| (&transform).mul(vec3::from(position).append(1.0)).xyz())
			.collect_vec();

		let normals = reader
			.read_normals()
			.ok_or_else(|| anyhow!("no normals"))?
			.map(vec3::from)
			.map(|normal| (&transform).mul(vec3::from(normal).append(0.0)).xyz().normalized())
			.collect_vec();

		let tangents = || -> Option<(Vec<vec3>, Vec<vec3>)> {
			let tangent_u = reader
				.read_tangents()?
				.map(vec4::from)
				.map(|tangent| (&transform).mul(tangent.xyz().append(0.0)).xyz().normalized())
				.collect_vec();

			// https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html
			// The bitangent vectors MUST be computed by taking the cross product of the normal and tangent XYZ vectors
			// and multiplying it against the W component of the tangent:
			//     bitangent = cross(normal.xyz, tangent.xyz) * tangent.w.
			let tangent_handedness = reader.read_tangents()?.map(|t| t[3]);
			let tangent_v = normals //
				.iter()
				.zip(tangent_u.iter())
				.zip(tangent_handedness)
				.map(|((&n, &t), h)| h * n.cross(t))
				.collect_vec();

			Some((tangent_u, tangent_v))
		}();

		let texcoords = reader.read_tex_coords(0).ok_or_else(|| anyhow!("no tex coords"))?.into_f32().map(vec2::from).collect_vec();

		let lightcoords: Option<Vec<vec2>> = match reader.read_tex_coords(1) {
			Some(iter) => Some(iter.into_f32().map(vec2::from).collect()),
			None => None,
		};
		println!("{}↳⛛ primitive: {} triangles", padding(depth + 1), indices.len() / 3);
		if lightcoords.is_some() {
			println!("{}↳🙾 dedicated lightmap UVs", padding(depth + 1));
		}

		if tangents.is_none() {
			println!("{}↳️❗️ NO TANGENTS FOR {}", padding(depth + 1), mesh.name().unwrap_or_default());
		}

		// TODO: mesh stores only tangent_u (vec4), computes bitangent in vertex shader
		let (tangent_u, tangent_v) = match tangents {
			None => (None, None),
			Some((u, v)) => (Some(u), Some(v)),
		};

		let mesh = MeshBuffer2 {
			indices,
			positions,
			normals,
			texcoords,
			lightcoords,
			tangent_u,
			tangent_v,
		};

		result.push(Primitive { material, mesh })
	}

	Ok(result)
}

// Bounding box around faces.
// Used to convert scene extras like JumpPads, SpawnPoints, ...
// Their mesh won't show up in the Map, but is only used to compute a bounding box (position + size).
pub fn bounding_box(primitives: &[Primitive]) -> Result<BoundingBox32> {
	primitives
		.iter()
		.map(|Primitive { mesh, .. }| mesh.bounds().expect("mesh has vertices"))
		.reduce(|a, b| a.join(&b))
		.ok_or_else(|| anyhow!("boundingbox: mesh is empty"))
}

fn parse_gltf_material(material: &gltf::Material) -> Result<(Handle, MaterialDef)> {
	let mat_handle = Handle::from_str(material.name().unwrap_or(DEFAULT_MATERIAL))?;

	let def = MaterialDef {
		base_color: base_color(material).map_err(inspect_error).ok().unwrap_or(handle("#aaaaaa")), // TODO: handle missing
		normal_map: normal_map(material)?,
		emissive: parse_emissive(material)?,
	};
	Ok((mat_handle, def))
}

fn parse_emissive(material: &gltf::Material) -> Result<Option<EmissiveDef>> {
	// in GLTF, emissive, like base_color, is a poor-man's enum:
	//    emissive_texture OR
	//    emissive_factor (uniform color) OR
	//    neither

	// GLTF Material has "emissive_texture" set
	if let Some(emissive_texture) = material.emissive_texture() {
		if material.emissive_factor() != [1.0, 1.0, 1.0] {
			bail!("material {}: both emissive texture and base_factor (uniform color) are set.", material.name().unwrap_or(""))
		}
		return Ok(Some(EmissiveDef {
			emissive_texture: texture_for_source(emissive_texture.texture().source().source())?,
			emissive_strength: material.emissive_strength().unwrap_or(1.0),
		}));
	}

	// GLTF Material has "emissive_factor" set (means uniform color)
	if material.emissive_factor() != [0.0, 0.0, 0.0] {
		if material.emissive_texture().is_some() {
			bail!("material {}: both emissive texture and base_factor (uniform color) are set.", material.name().unwrap_or(""))
		}
		let [r, g, b] = material.emissive_factor();
		return Ok(Some(EmissiveDef {
			emissive_texture: uniform_material([r, g, b]),
			emissive_strength: material.emissive_strength().unwrap_or(1.0),
		}));
	}

	// None set
	Ok(None)
}

fn base_color(material: &gltf::Material) -> Result<Handle> {
	match material.pbr_metallic_roughness().base_color_texture() {
		None => Ok(uniform_material4(material.pbr_metallic_roughness().base_color_factor())),
		Some(info) => Ok(texture_for_source(info.texture().source().source())?),
	}
}

fn normal_map(material: &gltf::Material) -> Result<Option<Handle>> {
	match material.normal_texture() {
		None => Ok(None),
		Some(info) => Ok(Some(texture_for_source(info.texture().source().source())?)),
	}
}

fn texture_for_source(source: gltf::image::Source) -> Result<Handle> {
	let uri = match source {
		gltf::image::Source::Uri { uri, .. } => uri,
		gltf::image::Source::View { view, .. } => match view.buffer().source() {
			gltf::buffer::Source::Bin => bail!("pbr_metallic_roughness: base_color_texture: `bin` is unsupported"),
			gltf::buffer::Source::Uri(uri) => uri,
		},
	};
	let uri = uri.rsplit_once(".").map(|(stem, _ext)| stem).unwrap_or(uri); // strip extension (png/jpg) used by Blender, the texture loader gets to decide that
	Handle::from_str(uri)
}

/// Turn f32 r,g,b into hex #rgb
fn uniform_material(factor: [f32; 3]) -> Handle {
	let [r, g, b] = factor.map(linear_to_srgb); // <<< probably wrong!
	Handle::from_str(&format!("#{r:02x}{g:02x}{b:02x}")).unwrap()
}

fn uniform_material4(factor: [f32; 4]) -> Handle {
	let [r, g, b, _] = factor;
	uniform_material([r, g, b])
}

fn load_map_gltf(map_dir: &MapDir) -> Result<(gltf::Document, Vec<gltf::buffer::Data>)> {
	// Load ".gltf" file, and corresponding ".bin" buffers from the same directory,
	// but not the images (these are handled separately by the texture cache, and not located next to the gltf file).
	println!("loading {:?}", map_dir);
	let path = map_dir.gltf_file();
	let base = path.parent().unwrap_or_else(|| Path::new("./"));
	let gltf::Gltf { document, .. } = gltf::Gltf::from_reader(open(&path)?).with_context(|| anyhow!("load GLTF {path:?}"))?;
	let blob = gltf::import_buffers(&document, Some(base), None).with_context(|| anyhow!("load BIN for {path:?}"))?;
	Ok((document, blob))
}

fn get_single_scene(gltf: &gltf::Document) -> Result<gltf::Scene> {
	if gltf.scenes().len() != 1 {
		bail!("GLTF file must contain one scene, have {}", gltf.scenes().len());
	}
	let scene = gltf.scenes().next().ok_or_else(|| anyhow!("no scene"))?;
	println!(" 🔶 Scene: {}", scene.name().unwrap_or("<Unnamed>"));
	Ok(scene)
}

pub fn padding(depth: u32) -> &'static str {
	&"                "[..((depth * 2).clamp(0, 16) as usize)]
}
