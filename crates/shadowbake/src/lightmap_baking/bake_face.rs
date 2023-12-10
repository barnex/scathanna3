use crate::prelude::*;

/// Triangular face for lightmap baking.
#[derive(Clone, Default, Serialize, Deserialize)]
pub(crate) struct BakeFace {
	// Vertex ordering has special meaning:
	//
	//   v0: v1+tangent1
	//    ^
	//    |
	//    |
	//    +--------> v2: v1+tangent2
	//    v1:origin
	//
	pub vertices: [vec3; 3],
	pub normals: [vec3; 3],
	pub texcoords: [vec2; 3],
	pub lightcoords: [vec2; 3],
	pub material: Handle,
	pub lightmap: Handle,
}

impl BakeFace {
	pub fn origin(&self) -> vec3 {
		self.vertices[1]
	}

	/// Tangent vectors, not scaled to unit length.
	pub fn sized_tangents(&self) -> [vec3; 2] {
		[self.vertices[0] - self.vertices[1], self.vertices[2] - self.vertices[1]]
	}

}

//-------------------------------------------------------------------------------- ray tracing

impl Bounded for BakeFace {
	fn bounds(&self) -> BoundingBox<f32> {
		BoundingBox::from_points(self.vertices.iter().copied()).expect("face has vertices")
	}
}

impl Intersect for BakeFace {
	type Attrib = (Normal, TexCoord, LightCoord, MaterialHandle, LightmapHandle); // normal, UV, material

	#[inline]
	fn intersect(&self, r: &Ray32, hr: &mut HitRecord<f32, Self::Attrib>) -> bool {
		let o = self.origin();
		let [a, b] = self.sized_tangents();

		let dir = r.dir;
		let n = a.cross(b);

		// TODO: early return for backface?
		// dir dot n => 0

		let s = r.start - o;
		let t = -n.dot(s) / n.dot(dir);
		//let n2 = n.dot(n);

		// handles NaN gracefully
		if !(t > 0.0 && t < hr.t) {
			return false;
		}

		let p = r.at(t) - o;

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
			let l = [l2,l1,l3]; // 👈 l1 belongs to o. (a, o, b) => (l2, l1, l2) .
			let normal = barycentric_interpolation(&self.normals, &l).normalized();
			let texcoord = barycentric_interpolation(&self.texcoords, &l);
			let lightcoord = barycentric_interpolation(&self.lightcoords, &l);
			hr.record(t, &(normal, texcoord, lightcoord, self.material, self.lightmap))
		}

		inside
	}
}

/*
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
		let t = BakeFace::triangle(handle(""), vec3(1.0, 2.0, -1.0), vec3(3.0, 2.0, -1.0), vec3(3.0, 4.0, -1.0));
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
*/