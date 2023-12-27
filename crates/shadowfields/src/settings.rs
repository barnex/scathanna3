use super::prelude::*;

/// User settings, as defined by "settings.toml".
#[derive(Clone)]
pub struct Settings {
	pub graphics: GraphicsOpts,
	pub controls: Controls,
	pub player: PlayerOpts,
	pub sound: SoundOpts,
	pub network: NetworkOpts,
	pub debug: DebugOpts,
	pub bot: BotSettings,
}

impl Settings {
	pub fn load(file_name: &str) -> Result<Self> {
		let settings_file = assets_dir().settings_file(file_name)?;
		let t: SettingsToml = load_toml(&settings_file)?;
		Ok(Self {
			graphics: t.graphics,
			controls: t.controls,
			player: t.player,
			sound: t.sound,
			network: t.network,
			debug: t.debug,
			bot: t.bot,
		})
	}
}

/// All user-controlled settings, read from "settings.toml".
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct SettingsToml {
	pub graphics: GraphicsOpts,
	pub controls: Controls,
	pub player: PlayerOpts,
	pub sound: SoundOpts,
	pub network: NetworkOpts,
	#[serde(default)]
	pub debug: DebugOpts,
	#[serde(default)]
	pub bot: BotSettings,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Controls {
	pub forward: char,
	pub left: char,
	pub backward: char,
	pub right: char,
	pub crouch: char,
	pub mouse_sensitivity: f32,
	pub mouse_stutter_filter: u8,
	pub mouse_smoothing: f32,
}

impl Default for Controls {
	fn default() -> Self {
		Self {
			forward: 'w',
			left: 'a',
			backward: 's',
			right: 'd',
			crouch: 'z',
			mouse_sensitivity: 100.0,
			mouse_stutter_filter: 1,
			mouse_smoothing: 0.0,
		}
	}
}

#[derive(Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct PlayerOpts {
	pub name: String,
	pub avatar: u8,
	pub team: String,
	#[serde(default)]
	pub advantage: bool,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct SoundOpts {
	pub enabled: bool,
	pub music: bool,
}

impl Default for SoundOpts {
	fn default() -> Self {
		Self { enabled: true, music: false }
	}
}

#[derive(Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct NetworkOpts {
	pub servers: Vec<String>,
}

#[derive(Deserialize, Clone)]
#[serde(default)]
pub struct DebugOpts {
	pub profile: bool,
	pub dbg_overlay: bool,
	pub bot_overlay: bool,
	pub fps_overlay: bool,
	pub ecs_overlay: bool,
	pub bb_overlay: bool,
	pub axes: bool,
	pub time_passage: f32,
	pub monitor_mouse: bool,
}

impl Default for DebugOpts {
	fn default() -> Self {
		Self {
			profile: default(),
			dbg_overlay: default(),
			bot_overlay: default(),
			fps_overlay: default(),
			ecs_overlay: default(),
			bb_overlay: default(),
			axes: default(),
			time_passage: 1.0,
			monitor_mouse: false,
		}
	}
}
