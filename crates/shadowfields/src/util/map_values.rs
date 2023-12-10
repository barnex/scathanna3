use super::internal::*;
use std::hash::Hash;

/// Extension trait for applying a function to the values of a collection. E.g.:
///
/// ```
/// 	# use shadowfields::util::*;
/// 	# use std::collections::HashMap;
///
/// 	let a = vec![1, 2, 3];
/// 	let b = a.map_values(|v|v as f32);
///     assert_eq!(b, vec![1.0, 2.0, 3.0]);
///
/// 	let mut a = HashMap::default();
/// 	a.insert(0, "foo");
/// 	a.insert(1, "bar");
/// 	let b = a.map_values(|v|v.to_owned() + v);
///		assert_eq!(b[&0], "foofoo");
///		assert_eq!(b[&1], "barbar");
/// ```
///
/// ☠️ 🪲 takes ownership. Instead implement map_second on key-value pairs, collect_map
pub trait MapValuesExt<V, W> {
	type Output;

	fn map_values<F>(self, f: F) -> Self::Output
	where
		F: Fn(V) -> W;
}

/// apply a function to the values of a HashMap.
impl<K, V, W> MapValuesExt<V, W> for HashMap<K, V>
where
	K: Eq + Hash,
{
	type Output = HashMap<K, W>;

	fn map_values<F>(self, mut f: F) -> Self::Output
	where
		F: FnMut(V) -> W,
	{
		self.into_iter().map(|(k, v)| (k, f(v))).collect()
	}
}
