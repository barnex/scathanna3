use super::internal::*;

type Result<T, E> = std::result::Result<T, E>;

pub(crate) struct Cache<T> {
	inner: RefCell<HashMap<Handle, Option<Arc<T>>>>,
}

pub(crate) trait Load: Sized {
	fn load(name: &str) -> Result<Self, anyhow::Error>;
}

impl<T> Default for Cache<T> {
	fn default() -> Self {
		Self { inner: default() }
	}
}

impl<T> Cache<T> {
	/// Fetch from cache. Don't load missing values (assumed pre-populated).
	/// Silently ignore errors (except in debug mode).
	pub fn try_get_noload(&self, handle: Handle) -> Option<Arc<T>> {
		let inner = self.inner.borrow();
		let v = inner.get(&handle);
		let v = v.as_ref().and_then(|v| v.as_ref());
		#[cfg(debug_assertions)]
		if v.is_none() {
			panic!("BlockingCache: {:?} not found", handle);
		}
		v.map(Arc::clone)
	}

	pub fn load_sync_with<F: Fn(&str) -> Result<T, anyhow::Error>>(&self, load: F, handle: Handle) -> Option<Arc<T>> {
		self.inner
			.borrow_mut()
			.entry(handle)
			.or_insert_with(|| {
				load(handle.as_str()) //
					.map_err(|e| {
						let typename = std::any::type_name::<T>().split("::").last().unwrap_or_default();
						LOG.write(format!("[debug_assertion] Error loading {} `{handle}`: {e:#}", typename));
						#[cfg(debug_assertions)]
						panic!("[debug_assertion] Error loading {} `{handle}`: {e:#}", typename);
					})
					.ok()
					.map(Arc::new)
			})
			.clone()
	}
}

impl<T: Load> Cache<T> {
	pub fn load_sync(&self, handle: Handle) -> Option<Arc<T>> {
		self.load_sync_with(|name| T::load(name), handle)
	}
}

impl Cache<Texture> {
	/// Load texture (synchronous), or return fallback texture
	pub fn load_or_default(&self, handle: Handle) -> Arc<Texture> {
		self.load_sync(handle).unwrap_or_else(|| ctx().fallback_texture.clone())
	}
}

#[derive(Debug, Clone)]
pub struct CacheErr;

impl std::fmt::Display for CacheErr {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "cache load error")
	}
}

impl std::error::Error for CacheErr {}
