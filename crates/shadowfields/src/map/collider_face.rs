use super::internal::*;

/////////////////////////////////////////////////////
/////////////////////////////////////////////////////
/////////////////////////////////////////////////////
/// TODO: use triangle instead. Remove!!
/////////////////////////////////////////////////////
/////////////////////////////////////////////////////
/////////////////////////////////////////////////////

/// A Rectangular or Triangular shape + material ID.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ColliderFace {
	// Vertex ordering has special meaning:
	//
	//   v0: v1+tangent1
	//    ^        * (v3 not stored)
	//    |
	//    |
	//    +--------> v2: v1+tangent2
	//    v1:origin
	//
	pub vert: [vec3; 3],
	//pub normals: [vec3; 3],
	//pub texcoords: [vec2; 3],
	//pub shape: FaceShape,
	pub material: Handle, // TODO: attr
}

impl ColliderFace {
	pub fn new(vert: [vec3; 3], mat: Handle) -> Self {
		Self { vert, material: mat }
	}

	// ???
	pub fn origin(&self) -> vec3 {
		self.vert[1]
	}

	/// 3D position corresponding to a UV coordinate inside the face.
	#[inline]
	pub fn pos_for_uv(&self, uv: vec2) -> vec3 {
		let o = self.origin();
		let [a, b] = self.sized_tangents();
		let (u, v) = uv.into();
		o + (u * a) + (v * b)
	}

	/// Normal vector, not scaled to unit length.
	pub fn sized_normal(&self) -> vec3 {
		let a = self.vert[1] - self.vert[0];
		let b = self.vert[2] - self.vert[0];
		a.cross(b) // TODO: check handedness
	}

	/// Normal vector, scaled to unit length.
	pub fn normalized_normal(&self) -> vec3 {
		self.sized_normal().normalized()
	}

	/// Tangent vectors, not scaled to unit length.
	pub fn sized_tangents(&self) -> [vec3; 2] {
		[self.vert[0] - self.vert[1], self.vert[2] - self.vert[1]]
	}

	/// Tangent vectors, scaled to unit length.
	pub fn normalized_tangents(&self) -> [vec3; 2] {
		self.sized_tangents().map(|v| v.normalized())
	}

	/// A copy of with function `f` applied to all vertex positions.
	#[must_use = "does not alter self"]
	pub fn map_positions<F>(&self, f: F) -> Self
	where
		F: Fn(vec3) -> vec3,
	{
		let mut v = self.vert;
		v.iter_mut().for_each(|v| *v = f(*v));
		Self { vert: v, ..*self }
	}

	pub fn foreach_position<F>(&mut self, f: F)
	where
		F: Fn(vec3) -> vec3,
	{
		self.vert.iter_mut().for_each(|ptr| *ptr = f(*ptr))
	}
}

//-------------------------------------------------------------------------------- ray tracing

impl Bounded for ColliderFace {
	fn bounds(&self) -> BoundingBox<f32> {
		BoundingBox::from_points(self.vert.iter().copied()).expect("face has vertices")
	}
}

impl Intersect for ColliderFace {
	type Attrib = (vec3, vec2, Handle); // normal, UV, material

	#[inline]
	fn intersect(&self, r: &Ray32, hr: &mut HitRecord<f32, Self::Attrib>) -> bool {
		self.intersect_triangle(r, hr)
	}
}

impl ColliderFace {
	#[inline]
	fn intersect_triangle(&self, r: &Ray32, hr: &mut HitRecord<f32, (vec3, vec2, Handle)>) -> bool {
		let o = self.origin();
		let [a, b] = self.sized_tangents();

		let dir = r.dir;

		let n = a.cross(b);

		let s = r.start - o;
		let t = -n.dot(s) / n.dot(dir);
		//let n2 = n.dot(n);

		// handles NaN gracefully
		if !(t > 0.0 && t < hr.t) {
			return false;
		}

		let p = r.at(t) - o;
		// TODO: s + r.dir * t;

		// Barycentric coordinates for 3D triangle, after
		// Peter Shirley, Fundamentals of Computer Graphics, 2nd Edition.
		let nc = a.cross(p);
		let na = (b - a).cross(p - a);
		let n2 = n.dot(n);
		let l1 = n.dot(na) / n2;
		let l3 = n.dot(nc) / n2;
		let l2 = 1.0 - l1 - l3;

		let inside = f32::partial_min(f32::partial_min(l1, l2), l3) > 0.0;

		if inside {
			// TODO: check if l1, l2 are the correct barycentric coordinates!
			hr.record(t, &(self.normalized_normal(), vec2(l1, l2), self.material))
		}

		inside
	}
}

#[cfg(test)]
mod test {
	use super::*;

	/*

			 * (3,4)
			/|
		   / |
		  /  |
	(1,2)*---* (3,2)

	*/
	#[test]
	fn intersects() {
		let t = ColliderFace::new([vec3(1.0, 2.0, -1.0), vec3(3.0, 2.0, -1.0), vec3(3.0, 4.0, -1.0)], handle(""));
		let ez = vec3::EZ;

		assert!(!t.intersects(&Ray::new(vec3(0., 0., 0.,), -ez)));
		assert!(!t.intersects(&Ray::new(vec3(0., 0., 0.,), ez)));
		assert!(t.intersects(&Ray::new(vec3(2., 3., 0.,), -ez)));
		assert!(!t.intersects(&Ray::new(vec3(2., 3., 0.,), ez)));
		assert!(!t.intersects(&Ray::new(vec3(4., 3., 0.,), -ez)));
		assert!(!t.intersects(&Ray::new(vec3(4., 3., 0.,), ez)));
		assert!(!t.intersects(&Ray::new(vec3(2., -3., 0.,), -ez)));
		assert!(!t.intersects(&Ray::new(vec3(2., -3., 0.,), ez)));
		assert!(!t.intersects(&Ray::new(vec3(0., 0., -2.,), ez)));
		assert!(!t.intersects(&Ray::new(vec3(0., 0., -2.,), -ez)));
		assert!(t.intersects(&Ray::new(vec3(2., 3., -2.,), ez)));
		assert!(!t.intersects(&Ray::new(vec3(2., 3., -2.,), -ez)));
	}
}
