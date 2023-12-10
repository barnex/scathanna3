use super::internal::*;

/// Mesh + single material (similar to GLTF Primitive).
#[derive(Serialize, Deserialize, Clone)]
pub struct Primitive {
	pub material: Handle,
	pub mesh: MeshBuffer2,
}
