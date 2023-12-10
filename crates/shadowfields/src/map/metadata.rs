use super::internal::*;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Metadata {
	#[serde(default)]
	pub spawn_points: Vec<SpawnPoint>,

	#[serde(default)]
	pub pickup_points: Vec<PickupPoint>,

	#[serde(default)]
	pub jump_pads: Vec<JumpPad>,

	#[serde(default)]
	pub sun_def: Option<SunDef>,

	#[serde(default)]
	pub sky_color: vec3,

	#[serde(default)]
	pub sky_box: Option<String>,

	#[serde(default)]
	pub point_lights: Vec<PointLightDef>,

	#[serde(default)]
	pub materials: MaterialPalette,
}

impl Metadata {
	pub fn save(&self, map_dir: &MapDir) -> Result<()> {
		let file = map_dir.metadata_file();
		println!("saving {file:?}");
		let mut file = create(&file)?;
		Ok(ron::ser::to_writer_pretty(&mut file, self, default())?)
	}

	pub fn load(map_dir: &MapDir) -> Result<Self> {
		let mut file = open(&map_dir.metadata_file())?;
		Ok(ron::de::from_reader(&mut file)?)
	}
}
