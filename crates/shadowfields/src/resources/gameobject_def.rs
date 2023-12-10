// use crate::internal::*;
// 
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
// pub struct GameObjectDef {
// 	pub mesh: Handle,
// 	pub mesh_scale: f32, // <----- TODO: not consistently used
// 	pub material: Handle,
// 	pub aabb_size: vec3,
// 	pub on_collide: Option<Handle>,
// }
// 
// impl GameObjectDef {
// 	pub fn load(name: &str) -> Result<Self> {
// 		let def: iofmt::Object = load_ron(&assets_dir().find_object(name)?)?;
// 
// 		let aabb_size = if def.aabb_size == [0.0; 3] { vec3::repeat(def.scale) } else { vec3::from(def.aabb_size) };
// 		let on_collide = if def.on_collide.is_empty() { None } else { Some(def.on_collide.parse()?) };
// 
// 		Ok(Self {
// 			mesh: def.mesh.parse()?,
// 			material: def.material.parse()?,
// 			mesh_scale: def.scale,
// 			aabb_size,
// 			on_collide,
// 		})
// 	}
// }
// 
// // private namespace to avoid "Object" naming conflict.
// // used for RON deserialization only.
// mod iofmt {
// 	use super::*;
// 
// 	/// Schema for assets/objects/*.ron files.
// 	#[derive(Deserialize, Debug)]
// 	pub struct Object {
// 		pub mesh: String,
// 
// 		#[serde(default = "one")]
// 		pub scale: f32,
// 
// 		#[serde(default = "fallback")]
// 		pub material: String,
// 
// 		#[serde(default)]
// 		pub aabb_size: [f32; 3],
// 
// 		#[serde(default)]
// 		pub on_collide: String,
// 	}
// 
// 	fn one() -> f32 {
// 		1.0
// 	}
// 
// 	fn fallback() -> String {
// 		"fallback".into()
// 	}
// }