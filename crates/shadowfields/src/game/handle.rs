use super::internal::*;

/// Short string value (Copy type) used to identify assets.
///
/// E.g.:
/// 	handle("blue_crate")
///
/// Handles are small enough to be used as if they were numerical handles.
#[derive(Eq, PartialEq, Clone, Copy, Hash, Serialize, Deserialize)]
pub struct Handle([u8; 31]);

impl FromStr for Handle {
	type Err = Error;
	fn from_str(s: &str) -> Result<Self> {
		let src = s.as_bytes();
		let mut bytes = [0u8; 31];
		if src.len() > bytes.len() {
			bail!("handle too long: {s}, must be <= {} characters", bytes.len())
		}
		let n = usize::min(src.len(), bytes.len());
		bytes[..n].clone_from_slice(&src[..n]);
		Ok(Handle(bytes))
	}
}

impl Handle {
	fn len(&self) -> usize {
		self.0.iter().filter(|&v| *v != 0u8).count()
	}

	pub fn as_str(&self) -> &str {
		std::str::from_utf8(&self.0[..self.len()]).unwrap()
	}
}

impl std::fmt::Display for Handle {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl std::fmt::Debug for Handle {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self)
	}
}

impl Default for Handle {
	fn default() -> Self {
		Self([0; 31])
	}
}

/// Handle from hard-coded string.
/// Panics if `name` is too long.
/// TODO: make this a macro with compile-time checking instead of panic:
/// 	handle!("tnt_crate")
pub fn handle(name: &'static str) -> Handle {
	name.parse().expect("invalid handle")
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_handle() {
		let h = handle("cube");
		println!("{h}");
		let j = serde_json::to_string(&h).unwrap();
		println!("{j}")
	}
}
