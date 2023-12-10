use clap::Parser;
use shadowfields::prelude::*;

/// Play the game by connecting to a server.
#[derive(Parser)]
struct PlayFlags {
	/// Force connect to this server, instead of using settings.toml.
	#[arg(long)]
	server: Option<String>,

	/// Path to alternative `settings.toml` file
	#[arg(long, default_value = "settings.toml")]
	settings: String,

	/// Override player name.
	#[arg(short, long)]
	name: Option<String>,

	/// Override team (red|green|blue)
	#[arg(short, long)]
	team: Option<String>,

	/// Override avatar (red|green|blue)
	#[arg(short, long)]
	avatar: Option<u8>,

	/// Overrides settings.toml
	#[arg(long)]
	mouse_sensitivity: Option<f32>,

	/// Force disable textures (overrides settings.toml).
	#[arg(long)]
	textures: Option<bool>,

	/// Force lightmap nearest filtering (overrides settings.toml).
	#[arg(long)]
	lightmap_nearest: Option<bool>,

	/// Overrides settings.toml
	#[arg(long)]
	fullscreen: Option<bool>,

	/// Overrides settings.toml
	#[arg(long)]
	msaa: Option<bool>,

	/// Overrides settings.toml
	#[arg(long)]
	anisotropy: Option<u8>,

	/// Overrides settings.toml
	#[arg(long)]
	normal_maps: Option<bool>,

	/// Overrides settings.toml
	#[arg(long)]
	vsync: Option<bool>,

	/// Force disable sound (overrides settings.toml).
	#[arg(long)]
	sound: Option<bool>,

	/// Show frames per second (overrides settings.toml).
	#[arg(long)]
	fps: Option<bool>,

	/// Save timing profile (overrides settings.toml).
	#[arg(long)]
	profile: Option<bool>,

	/// Player is a bot
	#[arg(long)]
	bot: Option<bool>,
}

fn main() -> Result<()> {
	env_logger::builder().format_timestamp(Some(env_logger::TimestampPrecision::Millis)).init();
	print_debug_warning();

	let flags = PlayFlags::parse();
	init_assets_dir("assets")?;
	let settings = Settings::load(&flags.settings).context("load settings")?;
	let settings = override_play_settings(settings, flags);

	exit_on_error(shadowfields::play_main(settings))
}

fn override_play_settings(mut settings: Settings, flags: PlayFlags) -> Settings {
	if let Some(server) = flags.server {
		settings.network.servers = vec![server];
	}

	flag_override(&mut settings.controls.mouse_sensitivity, flags.mouse_sensitivity);
	flag_override(&mut settings.graphics.anisotropy, flags.anisotropy);
	flag_override(&mut settings.graphics.fullscreen, flags.fullscreen);
	flag_override(&mut settings.graphics.lightmap_nearest, flags.lightmap_nearest);
	flag_override(&mut settings.graphics.msaa, flags.msaa);
	flag_override(&mut settings.graphics.normal_maps, flags.normal_maps);
	flag_override(&mut settings.graphics.textures, flags.textures);
	flag_override(&mut settings.graphics.vsync, flags.vsync);
	flag_override(&mut settings.player.name, flags.name);
	flag_override(&mut settings.player.team, flags.team);
	flag_override(&mut settings.player.avatar, flags.avatar);
	flag_override(&mut settings.sound.enabled, flags.sound);
	flag_override(&mut settings.debug.fps_overlay, flags.fps);
	flag_override(&mut settings.debug.profile, flags.profile);
	flag_override(&mut settings.bot.enabled, flags.bot);

	settings
}
