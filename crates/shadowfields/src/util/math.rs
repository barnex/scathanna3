use super::internal::*;
use std::ops::Mul;

pub use std::f32::consts::PI;
#[allow(unused)]
pub const DEG: f32 = PI / 180.0;

#[allow(unused)]
pub const INF: f32 = f32::INFINITY;

/// Shorthand for `if cond {true_value}else{false_value}`.
pub fn select<T>(cond: bool, true_value: T, false_value: T) -> T {
	match cond {
		true => true_value,
		false => false_value,
	}
}

/*
/// Sample a random unit vector with isotropic direction.
pub fn sample_isotropic_direction(rng: &mut impl rand::Rng) -> vec3 {
	let norm = rand_distr::StandardNormal;
	vec3(rng.sample(norm), rng.sample(norm), rng.sample(norm)).normalized()
}
*/

pub fn square<T: Mul + Copy>(x: T) -> T::Output {
	x * x
}

/// Pretend `PartialOrd` is `Ord`, so we can compare `f32`s ignoring NaNs.
/// Usage:
/// 	foo.iter().max_by(partial_max)
/// 	bar.sort_by(partial_max)
///
pub fn partial_max(a: &f32, b: &f32) -> std::cmp::Ordering {
	use std::cmp::Ordering::*;
	if a > b {
		return Greater;
	}
	if b > a {
		return Less;
	}
	if a == b {
		return Equal;
	}
	// Chosen so that NaNs never win the `max` comparison.
	if a.is_nan() {
		return Less;
	}
	if b.is_nan() {
		return Greater;
	}
	Equal
}

/// Linear interpolation ('mix') between `v0` and `v1`.
/// ```
/// # use vector::*;
/// # use shadowfields::util::*;
/// let a = 1.0;
/// let b = 2.0;
/// assert_eq!(lerp(a, b, 0.2), 1.2);
///
/// let a = vec2(1.0, 2.0);
/// let b = vec2(2.0, 4.0);
/// assert_eq!(lerp(a, b, 0.5), vec2(1.5, 3.0));
/// ```
#[inline]
pub fn lerp<V, T>(v0: V, v1: V, t: T) -> V
where
	V: Add<Output = V> + Sub<Output = V> + Mul<T, Output = V> + Clone,
{
	v0.clone() + (v1 - v0) * t
}

/// Trilinear interpolation.
/// `c` is indexed by `[z][y][x]`.
/// `t` must be between 0.0 and 1.0
/// https://en.wikipedia.org/wiki/Trilinear_interpolation
#[inline]
pub fn trilinear_interp<V, T>(c: &[[[V; 2]; 2]; 2], t: Vector3<T>) -> V
where
	V: Add<Output = V> + Sub<Output = V> + Mul<T, Output = V> + Copy,
	T: Copy,
{
	let c00 = lerp(c[0][0][0], c[0][0][1], t.x());
	let c01 = lerp(c[0][1][0], c[0][1][1], t.x());
	let c10 = lerp(c[1][0][0], c[1][0][1], t.x());
	let c11 = lerp(c[1][1][0], c[1][1][1], t.x());
	let c0 = lerp(c00, c01, t.y());
	let c1 = lerp(c10, c11, t.y());
	lerp(c0, c1, t.z())
}

/// Linear interpolation between points (x0, y0) and (x1, y1).
///         ^
///      y1 |-----------*
///  result |-------*   |
///      y0 |---*   |   |
///         |   |   |   |
///         +---+---+---+--->
///            x0   x  x1
#[inline]
pub fn linterp<X, Y>(x0: X, y0: Y, x1: X, y1: Y, x: X) -> Y
where
	X: Add<Output = X> + Sub<Output = X> + Div<X, Output = X> + Clone,
	Y: Add<Output = Y> + Sub<Output = Y> + Mul<Y, Output = Y> + Mul<X, Output = Y> + Clone,
{
	let t = (x - x0.clone()) / (x1 - x0);
	y0.clone() + (y1 - y0) * t
}

/// Wrap an angle (in radians) to an equivalent angle in the range -PI..PI.
pub fn wrap_angle(angle: f32) -> f32 {
	if angle > PI {
		return angle - 2.0 * PI;
	}
	if angle < -PI {
		return angle + 2.0 * PI;
	}
	angle
}

pub fn approx_eq(a: f32, b: f32) -> bool {
	(a - b).abs() < 1e-5
}

//#[inline]
//pub fn and(a: bool, b: bool) -> bool {
//	a && b
//}

//pub fn is_aligned_to(v: ivec3, align: u32) -> bool {
//	let align = align as i32;
//	v.x() % align == 0 && v.y() % align == 0 && v.z() % align == 0
//}

//#[cfg(test)]
//mod test {
//
//	use super::*;
//
//	#[test]
//	fn test_is_aligned_to() {
//		assert_eq!(is_aligned_to(ivec3(-1, 0, 1), 1), true);
//		assert_eq!(is_aligned_to(ivec3(-1, 0, 1), 2), false);
//		assert_eq!(is_aligned_to(ivec3(-1, 0, 1), 4), false);
//		assert_eq!(is_aligned_to(ivec3(-33, 24, 789), 1), true);
//		assert_eq!(is_aligned_to(ivec3(-33, 24, 789), 2), false);
//		assert_eq!(is_aligned_to(ivec3(-4, 0, 8), 2), true);
//		assert_eq!(is_aligned_to(ivec3(-4, 0, 8), 4), true);
//		assert_eq!(is_aligned_to(ivec3(-4, 0, 8), 8), false);
//		assert_eq!(is_aligned_to(ivec3(4, 4, 4), 8), false);
//		assert_eq!(is_aligned_to(ivec3(-4, 0, 8), 16), false);
//		assert_eq!(is_aligned_to(ivec3(-40, 100, 80), 4), true);
//		assert_eq!(is_aligned_to(ivec3(-40, 100, 80), 8), false);
//		assert_eq!(is_aligned_to(ivec3(-4, 8, 4), 4), true);
//	}
//}
