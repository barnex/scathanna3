use crate::prelude::*;
use std::cmp::PartialOrd;

/// Axis Aligned Box, used to accelerate intersection tests with groups of objects.
/// See https://en.wikipedia.org/wiki/Minimum_bounding_box#Axis-aligned_minimum_bounding_box.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Bounds2D<T>
where
	T: Copy,
{
	pub min: Vector2<T>,
	pub max: Vector2<T>,
}
impl<T> Bounds2D<T>
where
	T: Copy + PartialOrd + std::fmt::Debug,
{
	/// Bounding box containing all points with coordinates between `min` and `max`.
	/// `min`'s components must not be larger than `max`'s.
	#[inline]
	pub fn new(min: Vector2<T>, max: Vector2<T>) -> Self {
		//if !(min.zip(max, |min, max| (min, max)).iter().all(|(min, max)| min <= max)){
		//	dbg!(min, max);
		//}
		debug_assert!(min.zip(max, |min, max| (min, max)).iter().all(|(min, max)| min <= max));
		Self { min, max }
	}

	/// The bounding box's 4 corners, in CCW order.
	pub fn corners(&self) -> [Vector2<T>; 4] {
		let (x1, y1) = self.min.into();
		let (x2, y2) = self.max.into();
		[
			Vector2::new(x1, y1), //
			Vector2::new(x2, y1),
			Vector2::new(x2, y2),
			Vector2::new(x1, y2),
		]
	}

	pub fn map<F, U>(&self, f: F) -> Bounds2D<U>
	where
		F: Fn(T) -> U + Copy,
		U: Copy,
	{
		Bounds2D {
			min: self.min.map(f),
			max: self.max.map(f),
		}
	}
}

impl<T> Bounds2D<T>
where
	T: Float,
{
	/// Center position.
	/// ```
	/// # use shadowfields::raytrace::*;
	/// # use vector::*;
	/// let bb = Bounds2D::new(vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0));
	/// assert_eq!(bb.center(), vec3(2.5, 3.5, 4.5));
	/// ```
	pub fn center(&self) -> Vector2<T> {
		(self.min + self.max) / (T::ONE + T::ONE)
	}
}

impl<T> Bounds2D<T>
where
	T: Number,
{
	/// Size in each direction.
	/// ```
	/// # use shadowfields::raytrace::*;
	/// # use vector::*;
	/// let bb = Bounds2D::new(ivec3(1, 2, 3), ivec3(2, 4, 8));
	/// assert_eq!(bb.size(), ivec3(1, 2, 5));
	/// ```
	pub fn size(&self) -> Vector2<T> {
		self.max - self.min
	}

	/// Test if a point lies inside the bounding box
	/// (including its boundaries).
	pub fn contains(&self, point: Vector2<T>) -> bool {
		point.x() >= self.min.x() //.
		&& point.x() <= self.max.x()
		&& point.y() >= self.min.y()
		&& point.y() <= self.max.y()
	}
}

impl Bounds2D<f32> {
	/// Does a line segment `start..end` intersect this bounding box?
	/// Used for conservative rasterization.
	/// ```
	/// # use shadowfields::raytrace::*;
	/// # use vector::*;
	/// let bb = Bounds2D::new(ivec3(1, 2, 3), ivec3(2, 4, 8));
	/// assert_eq!(bb.size(), ivec3(1, 2, 5));
	/// ```
	#[inline]
	pub fn intersects_segment(&self, start: vec2, end: vec2) -> bool {
		let dir = (end - start).normalized();
		let tmin = (self.min - start).div2(dir);
		let tmax = (self.max - start).div2(dir);

		let ten3 = tmin.zip(tmax, f32::partial_min);
		let tex3 = tmin.zip(tmax, f32::partial_max);

		let ten = ten3.reduce(f32::partial_max);
		let tex = tex3.reduce(f32::partial_min);

		// `>=` aims to cover the degenerate case where
		// the box has size 0 along a dimension
		// (e.g. when wrapping an axis-aligned rectangle).
		if tex >= f32::partial_max(0.0, ten) {
			ten < (end - start).len()
		} else {
			false
		}
	}
}

impl Bounds2D<u32> {
	// Iterates over all points inside this rectangle, maximum *included*.
	pub fn iter_incl(&self) -> impl Iterator<Item = uvec2> {
		cross(self.min.x()..=self.max.x(), self.min.y()..=self.max.y()).map(|(x, y)| uvec2(x, y))
	}
}
