use crate::prelude::*;

pub(crate) struct Scene {
	pub opts: BakeOpts,
	pub gltf: ParsedGltf,
	pub faces: Node<BakeFace>,
	pub temp_lightmap: HashMap<LightmapHandle, Img<Color>>,

	// loaded textures
	pub base_colors: HashMap<MaterialHandle, Img<Color>>, // face material handle -> base color for indirect tinting
}

impl Scene {
	pub fn new(opts: BakeOpts, gltf: ParsedGltf, temp_lightmap: HashMap<Handle, Img<Color>>) -> Result<Self> {
		let faces = gltf
			.objects
			.iter()
			.map(|obj| obj.primitives.iter().map(|prim| (obj.name, prim)))
			.flatten()
			.map(|(name, prim)| {
				prim.mesh.iter_triangle_indices().map(move |tri| BakeFace {
					vertices: prim.mesh.triangle_positions(&tri),
					normals: prim.mesh.triangle_normals(&tri),
					texcoords: prim.mesh.triangle_texcoords(&tri),
					lightcoords: prim.mesh.triangle_lightcoords(&tri),
					material: prim.material,
					lightmap: name,
				})
			})
			.flatten()
			.collect_vec();

		let base_colors = faces
			.iter()
			.map(|face| face.material)
			.collect::<Set<_>>()
			.into_iter()
			.map(|mat| {
				gltf.metadata
					.materials
					.get(&mat)
					.map(|def| (mat, def.base_color))
					.ok_or_else(|| anyhow!("material not found in palette: {}", mat))
			})
			.collect::<Result<Vec<_>>>()?
			.into_par_iter()
			.map(|(mat, basecolor_handle)| load_image_or_color(basecolor_handle).map(|img| Img::from_srgb(&img)).map(|img| (mat, img)))
			.collect::<Result<_>>()?;

		Ok(Self {
			opts,
			gltf,
			faces: Node::build_tree(faces),
			temp_lightmap,
			base_colors,
		})
	}

}
