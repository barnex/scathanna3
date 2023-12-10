use super::internal::*;

/// TODO: rename DrawCmd
#[derive(Clone)]
pub(crate) struct Object {
	pub vao: Arc<VAO>,
	pub shader: Shader,
	pub index_range: Option<Range<u32>>,
}

impl Object {
	// TODO: take Arc<VAO>
	pub fn new(vao: Arc<VAO>, shader: Shader) -> Self {
		Self {
			vao,
			shader,
			index_range: None,
		}
	}
}
