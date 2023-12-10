use super::internal::*;

// Mesh(es) that share a single lightmap image.
// E.g. Ferris the crab model (use texture UVs as lightmap UVs),
// or a contiguous patch of floor (use dedicated lightmap UVs).
//
// Rename "LightmapGroup" / "Island" ?
//
#[derive(Serialize, Deserialize, Clone)]
pub struct GltfObject {
	/// Unique name, from Blender/GLTF, used to retreive corresponding lightmap images.
	pub name: Handle,

	/// Meshes by material handle
	pub primitives: Vec<Primitive>,
}
