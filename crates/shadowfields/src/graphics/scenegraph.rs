use super::internal::*;

#[derive(Clone)]
pub(crate) struct SceneGraph {
	pub viewport_size: uvec2,

	// WGPU uniforms (see global_uniforms.rs)
	pub bg_color: vec3,
	pub sun_dir: vec3,
	pub sun_color: vec3,
	pub camera: Camera,

	pub objects: Vec<Object>,
}

impl SceneGraph {
	pub fn new(viewport_size: uvec2) -> Self {
		Self {
			viewport_size,
			bg_color: BSOD_BLUE,
			sun_color: vec3::ONES,
			sun_dir: vec3(0.0, -1.0, 0.0),
			camera: default(),
			objects: default(),
		}
	}

	//pub fn clear(&mut self){
	//	self.objects.clear();
	//}

	pub fn push(&mut self, obj: Object) {
		self.objects.push(obj)
	}
}
