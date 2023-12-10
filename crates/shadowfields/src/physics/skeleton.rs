use super::internal::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub(crate) struct Skeleton {
	pub hsize: f32,
	pub vsize: f32,
	pub position: vec3, // center bottom
	pub velocity: vec3,
	pub orientation: Orientation,
}

// consider an entity standing "on the ground" if the floor is not more than this distance below.
pub const GROUND_PROBE_DIST: f32 = 0.05;

impl Skeleton {
	pub fn new(pos: vec3, orientation: Orientation, hsize: f32, vsize: f32) -> Self {
		Self {
			position: pos,
			hsize,
			vsize,
			orientation,
			velocity: default(),
		}
	}

	pub fn set_frame(&mut self, frame: Frame) {
		self.position = frame.position;
		self.velocity = frame.velocity;
		self.orientation = frame.orientation;
	}

	pub fn frame(&self) -> Frame {
		Frame {
			position: self.position,
			velocity: self.velocity,
			orientation: self.orientation,
		}
	}

	// bounding box for a player at position `pos`.
	pub fn bounds_for(&self, pos: vec3) -> BoundingBox<f32> {
		let min = pos - vec3(self.hsize / 2.0, 0.0, self.hsize / 2.0);
		let max = pos + vec3(self.hsize / 2.0, self.vsize, self.hsize / 2.0);
		BoundingBox::new(min, max)
	}

	pub fn bounds(&self) -> BoundingBox<f32> {
		self.bounds_for(self.position)
	}
}
