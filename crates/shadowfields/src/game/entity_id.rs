use super::internal::*;
use std::sync::atomic::AtomicUsize;

/// Entity ID.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Default, PartialOrd, Ord)]
pub(crate) struct ID(usize);

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

impl std::fmt::Display for ID {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "#{}", self.0)
	}
}

impl ID {
	/// A fresh, unique entity ID.
	pub fn new() -> Self {
		Self(NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
	}
}
