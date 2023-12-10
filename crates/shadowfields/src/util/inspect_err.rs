
pub trait InspectErr: Sized {
	type Error: std::fmt::Debug;
	fn inspect_err<F: FnOnce(&Self::Error)>(self, f: F) -> Self;

	fn log_err(self) -> Self {
		self.inspect_err(|e| log::error!("{e:#?}"))
	}
}

impl<T> InspectErr for anyhow::Result<T> {
	type Error = anyhow::Error;
	#[inline]
	fn inspect_err<F: FnOnce(&Self::Error)>(self, f: F) -> Self {
		if let Err(e) = &self {
			f(e)
		}
		self
	}
}
