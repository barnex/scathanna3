///! Custom properties modify the meaning of GLTF nodes during parsing (e.g. turn a cube into a jump pad).
use crate::prelude::*;

/// Union of all possible supported Blender custom properties (GLTF "extras").
/// Setting one of these custom properties modifies the meaning of a GLTF node during parsing (`parse_gltf`)
#[derive(Deserialize, Debug, Default, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct CustomProperties {
	/// Convert a node into a jump pad that propels items by the given height.
	/// The node's mesh gets hidden, and it's bounding box becomes the sensor
	/// (entering the box triggers the jump pad).
	pub jump_pad_height: Option<f32>,

	/// Convert a node into a spawn point for given team (0,1).
	/// (games without teams ignore the team and use all spawn points).
	pub spawn_point_team: Option<u8>,

	/// Conventionally attached to the "Sun" light node to indicate the sky (background) color.
	pub sky_color: Option<[f32; 3]>,

	/// Turn object into a pickup point for a given Prop. E.g.:
	/// 	"pickup": "shield",
	pub pickup: Option<String>,

	/// Object is deadly lava.
	pub lava: Option<bool>,

	/// blender hack: it's all to easy to accidentally add a custom property (defaults to "prop": 1.0).
	/// Ignore it for convenience.
	pub prop: Option<f32>,
}

impl CustomProperties {
	pub fn parse(extras: &gltf::json::Extras) -> Result<Self> {
		match extras {
			Some(raw_value) => Ok(serde_json::from_str(raw_value.get()).context("parse GLTF custom properties")?),
			None => Ok(default()),
		}
	}

	// Do these custom properties mean the mesh should be hidden?
	pub fn should_hide_mesh(&self) -> bool {
		match self {
			Self { lava: Some(true), .. } => false,
			_ => self.clone().with(|p| p.prop = None) != CustomProperties::default(),
		}
	}
}

// Apply custom properties to GLTF nodes (during parsing), modifying their meaning.
pub(crate) fn apply_custom_properties(parsed: &mut ParsedGltf, node: &gltf::Node, meshes: &[Primitive], custom_properties: &CustomProperties, depth: u32) -> Result<()> {
	let raw = node.extras().as_ref().map(|v| v.get()).unwrap_or_default();
	if !raw.is_empty() {
		println!("{}↳🔧 custom_properties: {}", padding(depth + 1), raw.chars().filter(|&c| c != '\n' && c != '\t').collect::<String>())
	}

	if custom_properties.spawn_point_team.is_some() {
		let position = bounding_box(meshes)?.center_bottom();
		parsed.metadata.spawn_points.push(SpawnPoint {
			position,
			yaw: default(), /*TODO*/
		});
		println!("{}↳👤 spawn point @{}", padding(depth + 1), position);
	}

	if let Some(jump_height) = custom_properties.jump_pad_height {
		println!("{}↳⏫ jump pad {}m", padding(depth + 1), jump_height);
		parsed.metadata.jump_pads.push(JumpPad {
			bounds: bounding_box(meshes)?,
			jump_height,
		});
	}

	const PICKUP_FREQUENCY: f32 = 10.0;
	if let Some(item) = &custom_properties.pickup {
		let position = bounding_box(meshes)?.center_bottom();
		println!("{}↳📤 pickup: {}", padding(depth + 1), item);
		parsed.metadata.pickup_points.push(PickupPoint {
			pos: position,
			item: item.parse()?,
			timer: Timer::one_off_ready(PICKUP_FREQUENCY),
		})
	}

	if let Some(sky_color) = custom_properties.sky_color {
		println!("{}↳⛅ sky_color: {:?}", padding(depth + 1), sky_color);
		parsed.metadata.sky_color = sky_color.into();
	}

	Ok(())
}
