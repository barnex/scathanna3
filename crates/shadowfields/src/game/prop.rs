use super::*;
use crate::prelude::*;

/// Props are small game objects like boxes, shields, hats,...
/// The server inserts/removes them from the Entity system.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct Prop {
	pub transform: Transform,
	pub mesh: Handle,
	pub texture: Handle,
	pub aabb_size: vec3,
	pub on_collide: Option<Handle>,
	pub pickup_point_id: Option<usize>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub(crate) struct PropDiff {
	pub transform: Option<Transform>,
	pub mesh: Option<Handle>,
}

impl PropDiff {
	pub fn apply_to(self, gobj: &mut Prop) {
		if let Some(v) = self.transform {
			gobj.transform = v;
		}
		if let Some(v) = self.mesh {
			gobj.mesh = v;
		}
	}
}

impl Prop {
	pub fn from_handle(handle: Handle) -> Result<Self> {
		Self::from_str(handle.as_str())
	}

	pub fn from_str(name: &str) -> Result<Self> {
		let default = Self {
			transform: default(),
			mesh: handle("box"),
			texture: handle("#777777"),
			aabb_size: vec3::ONES,
			on_collide: None,
			pickup_point_id: None,
		};

		Ok(match name {
			"cube" => Self {
				mesh: handle("box"),
				aabb_size: vec3::ONES,
				..default
			},
			"shield" => Self {
				mesh: handle("shield"),
				texture: handle("#7777FF"),
				aabb_size: vec3::ONES, // <<<<<
				on_collide: Some(handle("shield")),
				..default
			},
			"gift_box" => Self {
				mesh: handle("gift_box"),
				texture: handle("gift_box"),
				aabb_size: vec3::ONES, // <<<<<
				on_collide: Some(handle("gift_box")),
				..default
			},
			name => bail!("unknown prop: {name}"),
		})
	}

	pub fn position(&self) -> vec3 {
		self.transform.translation
	}

	pub fn center(&self) -> vec3 {
		self.transform.translation.with(|v| v[1] = self.aabb_size.y() / 2.0)
	}

	pub fn bounds(&self) -> BoundingBox<f32> {
		let pos = self.transform.translation;
		let size = self.aabb_size;
		BoundingBox {
			min: vec3(pos.x() - size.x() / 2.0, pos.y(), pos.z() - size.z() / 2.0),
			max: vec3(pos.x() + size.x() / 2.0, pos.y() + size.y(), pos.z() + size.z() / 2.0),
		}
	}
}
