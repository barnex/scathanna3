use super::prelude::*;

#[derive(Default)]
pub struct LogBuffer {
	lines: Mutex<Vec<String>>,
}

/// Global LogBuffer.
/// Text logged here appears in stdout and the game Shell.
pub static LOG: LogBuffer = LogBuffer::new();

impl LogBuffer {
	const MAX_LINES: usize = 128;

	pub const fn new() -> Self {
		Self { lines: Mutex::new(Vec::new()) }
	}

	pub fn write<T: AsRef<str>>(&self, line: T) {
		let line = line.as_ref();
		println!("{line}");

		let mut lines = self.lines.lock().expect("poisoned");

		// corner case: empty line
		if line.is_empty() {
			lines.push(line.into());
		}

		for line in line.lines() {
			lines.push(line.into());
			if lines.len() > Self::MAX_LINES {
				lines.remove(0);
			}
		}
	}

	pub fn tail(&self, n: usize) -> String {
		let lines = &self.lines.lock().unwrap();
		let mut res = String::new();
		let len = lines.len() as i32;
		let n = n as i32;
		for i in (len - n).clamp(0, len - 1)..len {
			res.push_str(&lines[i as usize]);
			res.push('\n');
		}
		res
	}

	pub fn replace_if_prefix<T: Into<String>>(&self, prefix: &str, line: T) {
		let line = line.into();
		let line = format!("{prefix} {line}");
		let mut lines = self.lines.lock().expect("poisoned");
		if let Some(last) = lines.last_mut() {
			if last.starts_with(prefix) {
				println!("{line}");
				*last = line
			} else {
				drop(lines);
				self.write(line)
			}
		} else {
			drop(lines);
			self.write(line)
		}
	}

	pub fn to_string(&self) -> String {
		let lines = self.lines.lock().expect("poisoned");
		lines.join("\n")
	}
}
