use super::internal::*;

// 2D Barycentric coordinates of a point with respect to a triangle with given vertices.
// https://en.wikipedia.org/wiki/Barycentric_coordinate_system
pub fn barycentric_coordinates(triangle_vertices: &[vec2; 3], point: vec2) -> [f32; 3] {
	let [(x1, y1), (x2, y2), (x3, y3)] = triangle_vertices.map(|v| v.tuple());
	let (x, y) = point.tuple();
	let det = (y2 - y3) * (x1 - x3) + (x3 - x2) * (y1 - y3);
	let lambda1 = ((y2 - y3) * (x - x3) + (x3 - x2) * (y - y3)) / det;
	let lambda2 = ((y3 - y1) * (x - x3) + (x1 - x3) * (y - y3)) / det;
	let lambda3 = 1.0 - lambda1 - lambda2;
	[lambda1, lambda2, lambda3]
}

// Does a 2D point lie inside a triangle defined by the given vertices?
pub fn is_inside(triangle_vertices: &[vec2; 3], point: vec2) -> bool {
	barycentric_coordinates(triangle_vertices, point).iter().all(|&v| v >= 0.0 && v <= 1.0)
}

// Interpolate between 3 vertex values, using the given weights.
// Weights are typically barycentric coordinates.
// (E.g. interpolate vertex colors inside a triangle)
#[inline]
pub fn barycentric_interpolation<T, U, V>(v: &[T; 3], weight: &[U; 3]) -> V
where
	T: Mul<U, Output = V> + Clone,
	U: Clone,
	V: Add<Output = V>,
{
	v[0].clone() * weight[0].clone() + v[1].clone() * weight[1].clone() + v[2].clone() * weight[2].clone()
}
