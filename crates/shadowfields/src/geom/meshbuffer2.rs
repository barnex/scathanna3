use super::internal::*;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct MeshBuffer2 {
	// Each run of 3 consecutive indices defines a triangle.
	pub indices: Vec<u32>,

	pub positions: Vec<vec3>,
	pub texcoords: Vec<vec2>,
	pub normals: Vec<vec3>,
	pub lightcoords: Option<Vec<vec2>>,
	pub tangent_u: Option<Vec<vec3>>,
	pub tangent_v: Option<Vec<vec3>>,
}

impl MeshBuffer2 {
	pub fn new() -> Self {
		Self::default()
	}

	/// Iterate over triangle indices (i.e. advance through indices in chunks of 3).
	pub fn iter_triangle_indices(&self) -> impl Iterator<Item = [u32; 3]> + '_ {
		debug_assert!(self.indices.len() % 3 == 0);
		self.indices.chunks_exact(3).map(|c| [c[0], c[1], c[2]])
	}

	/// Iterate over triangle vertex positions.
	pub fn iter_triangle_positions(&self) -> impl Iterator<Item = [vec3; 3]> + '_ {
		self.iter_triangle_indices().map(|tri| self.triangle_positions(&tri))
	}

	/// Given triangle vertex indices, return the corresponding vertex positions.
	pub fn triangle_positions(&self, tri: &[u32; 3]) -> [vec3; 3] {
		Self::map_tri(&self.positions, tri)
	}

	/// Given triangle vertex indices, return the corresponding normals.
	pub fn triangle_normals(&self, tri: &[u32; 3]) -> [vec3; 3] {
		Self::map_tri(&self.normals, tri)
	}

	pub fn triangle_tangents(&self, tri: &[u32; 3]) -> Option<[vec3; 3]> {
		self.tangent_u.as_ref().map(|tangents_u| Self::map_tri(tangents_u, tri))
	}

	pub fn triangle_bitangents(&self, tri: &[u32; 3]) -> Option<[vec3; 3]> {
		self.tangent_v.as_ref().map(|tangents_v| Self::map_tri(tangents_v, tri))
	}

	/// Given triangle vertex indices, return the lightmap coordinates.
	/// Uses the texture coordinates if no explicit lightmap coordinates are used.
	pub fn triangle_lightcoords(&self, tri: &[u32; 3]) -> [vec2; 3] {
		match &self.lightcoords.as_ref() {
			Some(lightcoords) => Self::map_tri(lightcoords, tri),
			None => self.triangle_texcoords(tri),
		}
	}

	/// Given triangle vertex indices, return the texture coordinates.
	pub fn triangle_texcoords(&self, tri: &[u32; 3]) -> [vec2; 3] {
		Self::map_tri(&self.texcoords, tri)
	}

	fn map_tri<T: Clone>(attr: &[T], tri: &[u32; 3]) -> [T; 3] {
		tri.map(|i| attr[i as usize].clone())
	}

	pub fn append(&mut self, rhs: MeshBuffer2) {
		if self.indices.is_empty() {
			*self = rhs;
			return;
		}

		let offset = self.positions.len() as u32;
		self.indices.extend(rhs.indices.iter().map(|v| v + offset));

		self.positions.extend_from_slice(&rhs.positions);
		self.texcoords.extend_from_slice(&rhs.texcoords);
		self.normals.extend_from_slice(&rhs.normals);

		self.lightcoords.as_mut().map(|v| v.extend_from_slice(&rhs.lightcoords.expect("lightcoords")));
		self.tangent_u.as_mut().map(|v| v.extend_from_slice(&rhs.tangent_u.expect("tangent_u")));
		self.tangent_v.as_mut().map(|v| v.extend_from_slice(&rhs.tangent_v.expect("tangent_v")));
	}

	pub(crate) fn convert_to_packed(&self) -> Result<MeshBuffer> {
		debug_assert!(self.positions.len() == self.normals.len());
		debug_assert!(self.positions.len() == self.texcoords.len());

		let n = self.positions.len();
		fn unwrap_or_zeros<T: Default + Clone>(v: &Option<Vec<T>>, n: usize) -> Vec<T> {
			match v {
				Some(v) => {
					debug_assert!(v.len() == n);
					v.clone()
				}
				None => std::iter::repeat(default()).take(n).collect(),
			}
		}

		/////////////////////////////////////////////////////////////////
		// TODO: don't zero out, error out or compute useful!
		/////////////////////////////////////////////////////////////////
		let lightcoords = self.lightcoords.as_ref().unwrap_or(&self.texcoords); // use texcoords as fallback
		let tangent_u = unwrap_or_zeros(&self.tangent_u, n);
		let tangent_v = unwrap_or_zeros(&self.tangent_v, n);

		let vertices = self
			.positions
			.iter()
			.enumerate()
			.map(|(i, &position)| VertexLM {
				position,
				texcoords: self.texcoords[i],
				normal: self.normals[i],
				lightcoords: lightcoords[i],
				tangent_u: tangent_u[i],
				tangent_v: tangent_v[i],
			})
			.collect();

		Ok(MeshBuffer {
			indices: self.indices.clone(),
			vertices,
		})
	}

	pub fn bounds(&self) -> Option<BoundingBox<f32>> {
		BoundingBox::from_points(self.positions.iter().copied())
	}
}
