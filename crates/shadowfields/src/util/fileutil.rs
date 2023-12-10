use super::internal::*;
use std::fs;

/// Prints error, if any. Fails on error in debug mode.
/// Usage:  fallible_stuff().map_err(inspect_error);
pub fn inspect_error(e: Error) -> Error {
	error!("{e}");

	#[cfg(debug_assertions)]
	panic!("debug: failing on error {e}");
	#[cfg(not(debug_assertions))]
	e
}

/// BufReader for reading file with more descriptive message on error.
pub fn open(file: &Path) -> Result<BufReader<File>> {
	log::info!("loading {}", file.to_string_lossy());
	Ok(BufReader::new(File::open(file).map_err(|err| anyhow!("open {:?}: {}", file, err))?))
}

/// Read a file entirely and return the contents.
pub fn read_file(file: &Path) -> Result<Vec<u8>> {
	let mut buf = Vec::new();
	open(file)?.read_to_end(&mut buf)?;
	Ok(buf)
}

/// Read a file entirely and return the contents.
pub fn read_to_string(file: &Path) -> Result<String> {
	let mut f = open(file)?;
	let mut buf = String::new();
	f.read_to_string(&mut buf)?;
	Ok(buf)
}

/// Deserialize a toml file.
pub fn load_toml<T: DeserializeOwned>(path: &Path) -> Result<T> {
	toml::from_str(&read_to_string(path)?).with_context(|| format!("load toml {path:?}"))
}

/// Deserialize a ron file.
pub fn load_ron<T: DeserializeOwned>(path: &Path) -> Result<T> {
	ron::from_str(&read_to_string(path)?).with_context(|| format!("load RON {path:?}"))
}

/// Serialize a ron file.
pub fn save_ron<T: Serialize>(path: &Path, value: &T) -> Result<()> {
	Ok(create(path)?.write_all(ron::ser::to_string_pretty(value, default())?.as_bytes())?)
}

/// BufWriter for writing file with more descriptive message on error.
/// Create parent directory if needed.
#[allow(dead_code)]
pub fn create(file: &Path) -> Result<BufWriter<File>> {
	log::info!("writing {}", file.to_string_lossy());
	if let Some(parent) = file.parent() {
		let _ = mkdir(parent);
	}
	Ok(BufWriter::new(File::create(file).map_err(|err| anyhow!("create {:?}: {}", file, err))?))
}

/// Read file names (no full path) in a directory.
pub fn read_dir_names(path: &Path) -> Result<impl Iterator<Item = PathBuf>> {
	Ok(fs::read_dir(path)
		.map_err(|e| anyhow!("read '{path:?}': {e}"))? //
		.filter_map(|entry| entry.ok())
		.map(|entry| PathBuf::from(entry.file_name())))
}

pub fn mkdir(path: impl AsRef<Path>) -> Result<()> {
	let path = path.as_ref();
	fs::create_dir(path).map_err(|e| anyhow!("create directory '{path:?}': {e}"))
}

/// Equivalent of "rm -rf": remove file/directory, succeed if it did not exist in the first place.
pub fn force_remove(path: impl AsRef<Path>) -> Result<()> {
	let path = path.as_ref();

	if !path.exists() {
		return Ok(());
	}

	if path.is_dir() {
		std::fs::remove_dir_all(path)?
	} else {
		std::fs::remove_file(path)?
	}

	match path.exists() {
		false => Ok(()),
		true => Err(anyhow!("failed to delete {path:?}")),
	}
}

pub fn save_bincode_gz<T>(data: &T, file: &Path) -> Result<()>
where
	T: Serialize,
{
	LOG.write(format!("saving {file:?}"));
	Ok(bincode::serialize_into(GzEncoder::new(create(file)?, flate2::Compression::best()), data)?)
}

pub fn load_bincode_gz<T>(file: &Path) -> Result<T>
where
	T: DeserializeOwned,
{
	LOG.replace_if_prefix("loading", file.to_string_lossy());
	Ok(bincode::deserialize_from(GzDecoder::new(open(file)?))?)
}
