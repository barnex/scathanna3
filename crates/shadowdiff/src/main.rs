use clap::Parser;
use shadowfields::prelude::*;

/// Convert GLTF files to game maps.
#[derive(Parser, Debug)]
pub struct Args {
	maps: Vec<String>,

	/// Populate missing golden data with current lightmaps.
	#[arg(short, long)]
	pub gold: bool,
}

fn main() -> Result<()> {
	env_logger::init();
	let args = Args::parse();
	init_assets_dir("assets")?;

	let test_maps = match args.maps.len() {
		0 => assets_dir().find_all_maps()?.into_iter().filter(|map| map.starts_with("test")).collect_vec(),
		_ => args.maps.clone(),
	};

	if test_maps.is_empty() {
		bail!("no test maps found")
	}

	let mut fail = vec![];
	for map_dir in &test_maps {
		println!("📁{map_dir}: ");
		match test(&args, map_dir) {
			Ok(msg) => println!("  ✅{msg}"),
			Err(e) => {
				println!("  ❌ {e}");
				fail.push(map_dir.clone());
			}
		}
	}

	match fail.len() {
		0 => {
			println!("✅ OK");
			Ok(())
		}
		n => Err(anyhow!("❌ {n} tests failed: {:?}", &fail)),
	}
}

fn test(args: &Args, map_name: &str) -> Result<String> {
	let map_dir = assets_dir().find_map_dir(map_name);
	let gold_dir = map_dir.lightmap_gold_dir();
	let lm_dir = map_dir.lightmap_dir();

	if !lm_dir.exists() {
		bail!("no lightmap dir")
	}

	if !gold_dir.exists() {
		match args.gold {
			false => return Ok("⚠️ SKIPPED: no golden data".into()),
			true => copy_dir(&gold_dir, &lm_dir)?,
		}
	}

	let mut max_diff = 0.0;
	for file in read_dir_names(&lm_dir)? //
		.filter(|p| !p.to_string_lossy().ends_with("_error.png"))
	{
		let diff = diff(&gold_dir.join(&file), &lm_dir.join(&file))?;
		max_diff = f64::max(max_diff, diff);
		if diff > 0.0 {
			println!("  🎇{file:?}: \tdiff = {diff:.3}");
		}
	}

	const TOLERANCE: f64 = 0.0;
	if max_diff > TOLERANCE {
		Err(anyhow!("diff too large: {max_diff} > {TOLERANCE}"))
	} else {
		Ok(format!("diff OK: {max_diff}"))
	}
}

fn diff(a: &Path, b: &Path) -> Result<f64> {
	let a = image::open(a).with_context(|| anyhow!("open {a:?}"))?.into_rgb8();
	let b = image::open(b).with_context(|| anyhow!("open {b:?}"))?.into_rgb8();

	if a.dimensions() != b.dimensions() {
		bail!("size mismatch: {:?} vs {:?}", a.dimensions(), b.dimensions());
	}

	let mut diff = 0.0;
	for (image::Rgb(a), image::Rgb(b)) in a.pixels().zip(b.pixels()) {
		for i in 0..3 {
			diff += f64::abs(a[i] as f64 - b[i] as f64) / 255.0;
		}
	}

	let (w, h) = a.dimensions();
	Ok(diff / (w * h) as f64)
}

fn copy_dir(gold_dir: &Path, lm_dir: &Path) -> Result<()> {
	println!("  📁cp {lm_dir:?}, {gold_dir:?}");
	mkdir(&gold_dir)?;
	for name in read_dir_names(lm_dir)? {
		println!("    🎇cp {:?} {:?}", lm_dir.join(&name), gold_dir.join(&name));
		std::fs::copy(lm_dir.join(&name), gold_dir.join(&name))?;
	}
	Ok(())
}
