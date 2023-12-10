use clap::Parser;
use shadowbake::prelude::*;
mod convergence_test;

/// Convert GLTF files to game maps.
#[derive(Parser, Debug)]
pub struct Flags {
	pub map_names: Vec<String>,

	/// Bake iteratively: gradually increase accuracy and save lightmaps regularly.
	#[arg(long, short)]
	pub iterative: bool,

	/// Do not re-bake (keep existing lightmaps), only rebuild mesh, metadata, etc.
	/// Useful when only pickups, spawn points, ... changed.
	#[arg(long, short)]
	pub no_bake: bool,

	//#[arg(long)]
	//pub convergence_test: bool,
	#[arg(long)]
	pub filter: Option<bool>,

	#[arg(long)]
	pub min_samples: Option<u32>,

	#[arg(long)]
	pub max_samples: Option<u32>,

	#[arg(long)]
	pub target_error: Option<f32>,
}

fn main() {
	exit_on_error(main_result())
}

fn main_result() -> Result<()> {
	env_logger::init();
	let flags = Flags::parse();

	if flags.map_names.is_empty() {
		bail!("need at least one map name")
	}

	init_assets_dir("assets")?;

	//if flags.convergence_test {
	//	return convergence_test(flags); //
	//}

	for map_name in &flags.map_names {
		bake_one(&flags, map_name)?;
	}

	Ok(())
}

fn bake_one(flags: &Flags, map_name: &str) -> Result<()> {
	println!("map: {}", &map_name);
	let map_dir = assets_dir().find_map_dir(&map_name);
	let bake_opts = load_bake_opts(&map_dir)?;
	let bake_opts = override_bake_settings(flags, bake_opts);
	match flags.no_bake {
		true => shadowbake::convert_only(&map_name, &bake_opts).map(|_| ()),
		false => shadowbake::convert_and_bake(&map_name, &bake_opts, flags.iterative),
	}
}

// Command-line flags override `baking.ron` settings,
// mainly for testing/experimentation.
fn override_bake_settings(flags: &Flags, mut opts: BakeOpts) -> BakeOpts {
	flag_override(&mut opts.filter, flags.filter);
	flag_override(&mut opts.min_samples, flags.min_samples);
	flag_override(&mut opts.max_samples, flags.max_samples);
	flag_override(&mut opts.target_error, flags.target_error);
	opts
}
