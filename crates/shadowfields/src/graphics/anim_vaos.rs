use super::internal::*;

/// List of Vertex Array Objects for linear morphing animations.
pub(crate) struct AnimVAOs {
	vaos: Vec<Arc<VAO>>,
}

impl AnimVAOs {
	pub fn new(vaos: Vec<Arc<VAO>>) -> Self {
		Self { vaos }
	}

	pub fn draw_animated(&self, tex: &Arc<Texture>, matrix: mat4, t: f32, bounding_box: &BoundingBox32, lightbox: &LightBox) -> Object {
		let ctx = ctx();
		debug_assert!(t >= 0.0 && t <= 1.0);

		let r = t * (self.vaos.len() as f32); // 0.0 .. 6.999..
		let i = r.floor() as usize; // 0 .. 6
		let t = r % 1.0; // 0.0 .. 0.9999...
				 //dbg!(r, i, t);
				 //println!();

		Object {
			vao: self.vaos[i].clone(), // TODO: defensive bound check.
			shader: ctx.shader_pack.animation(tex, matrix, t, bounding_box, lightbox),
			index_range: None,
		}
	}
}
