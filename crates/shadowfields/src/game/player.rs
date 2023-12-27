use super::internal::*;

/// Player data.
/// Part of the GameState.
/// Can be sent over the wire to communicate updates (e.g. position moved).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct Player {
	// controlled by server:
	pub id: ID, // uniquely identifies player across server and all clients

	pub name: String,  // nickname
	pub avatar_id: u8, // determines which avatar model is drawn (gl_client.rs).
	pub team: Team,
	pub spawned: bool, // playing or waiting for respawn?

	pub flying: bool,

	// controlled locally, synced to server:
	pub skeleton: Skeleton, // fully determines player position

	pub bump: Vector3<bool>,

	pub torso_size: vec3,
	pub head_size: vec3,
	pub cam_height: f32,
	pub walk_speed: f32,
	pub jump_speed: f32,
}

const ANIMAL_TORSO_SIZE: vec3 = vec3(0.6, 0.3, 0.6);
const ANIMAL_HEAD_SIZE: vec3 = vec3(0.6, 0.6, 0.6);

const HUMAN_TORSO_SIZE: vec3 = vec3(0.5, 0.9, 0.5);
const HUMAN_HEAD_SIZE: vec3 = vec3(0.5, 0.7, 0.5);

impl Player {
	pub fn new(id: ID, position: vec3, orientation: Orientation, name: String, avatar_id: u8, team: Team) -> Self {
		let (torso_size, head_size) = match avatar_id {
			0..=9 => (ANIMAL_TORSO_SIZE, ANIMAL_HEAD_SIZE),
			10.. => (HUMAN_TORSO_SIZE, HUMAN_HEAD_SIZE),
		};

		let max_width = f32::max(torso_size.x(), head_size.x());
		let total_height = torso_size.y() + head_size.y();

		// In games like UT, players can (single) jump slightly less than their own height.
		// Jumps typically last ~600ms.
		let jump_speed = jump_velocity_for_height(STD_GRAVITY, 1.0 * total_height);

		Self {
			id,
			name,
			avatar_id,
			spawned: false,
			team,
			skeleton: Skeleton::new(position, orientation, max_width, total_height),
			flying: false,
			torso_size,
			head_size,
			cam_height: torso_size.y() + head_size.y() / 3.0,
			walk_speed: 6.0,
			jump_speed,
			bump: default(),
		}
	}

	// __________________________________________________________________________________ control

	pub fn on_ground(&self, map: &Map) -> bool {
		!self.pos_ok(map, self.skeleton.target_position - vec3(0.0, GROUND_PROBE_DIST, 0.0))
	}

	/// TODO
	//pub fn set_orientation(&mut self, inputs: &Inputs, sens: f32) {
	//	let mouse_sens = 0.00001 * sens;
	//	self.skeleton.orientation.yaw = wrap_angle(self.skeleton.orientation.yaw - inputs.mouse_delta().x() * mouse_sens);
	//	self.skeleton.orientation.pitch = (self.skeleton.orientation.pitch + inputs.mouse_delta().y() * mouse_sens).clamp(-89.0 * DEG, 89.0 * DEG);

	//	//let a = 0.8;
	//	//self.skeleton.filtered_orientation.yaw = a * self.skeleton.filtered_orientation.yaw + (1.0 - a) * self.skeleton.orientation.yaw;
	//	//self.skeleton.filtered_orientation.pitch = a * self.skeleton.filtered_orientation.pitch + (1.0 - a) * self.skeleton.orientation.pitch;
	//}

	// __________________________________________________________________________________ accessors

	/// Center-bottom position of the bounding box.
	pub fn position(&self) -> vec3 {
		self.skeleton.filtered_position
	}

	/// Position right beneath player, used to check what they're standing on.
	pub fn ground_probe(&self) -> vec3 {
		self.skeleton.target_position - GROUND_PROBE_DIST * vec3::EY
	}

	pub fn center(&self) -> vec3 {
		self.skeleton.filtered_bounds().center()
	}

	pub fn orientation(&self) -> Orientation {
		self.skeleton.orientation
	}

	/// Ray looking through the player's crosshair.
	pub fn line_of_sight(&self) -> Ray64 {
		Ray64::new(self.camera().position.into(), self.orientation().look_dir().into())
	}

	pub fn camera(&self) -> Camera {
		Camera::default().with(|c| {
			c.position = self.skeleton.filtered_position + vec3(0.0, self.cam_height, 0.0);
			c.orientation = self.orientation();
			//c.orientation = self.skeleton.filtered_orientation;
			//TODO: mouse filter option + winit: poll twice
		})
	}

	/// Intersect ray with player hitbox.
	pub fn intersect(&self, ray: &Ray64) -> Option<f64> {
		// Cannot get hit if not spawned.
		match self.spawned {
			true => self.skeleton.filtered_bounds().convert::<f64>().intersect(ray),
			false => None,
		}
	}

	// is this player position allowed in the map?
	// I.e. not bumping into blocks.
	pub fn pos_ok(&self, map: &Map, pos: vec3) -> bool {
		debug_assert!(pos.is_finite());
		!map.bumps(&self.skeleton.bounds_for(pos))
	}
}
