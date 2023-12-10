use super::internal::*;
use std::process::exit;

pub fn exit_on_error(result: Result<(), anyhow::Error>) -> ! {
	match result {
		Err(e) => {
			eprintln!("ERROR: {e:#}");
			exit(1)
		}
		Ok(_) => exit(0),
	}
}

/// Panic on error if debug_assertions are enabled.
/// Log error otherwise.
pub fn debug_panic(err: Error){
	log::error!("{err:?}");
	#[cfg(debug_assertions)]
	panic!("{err:?}")
}

// ANSI clear line
pub const CLEAR: &str = "\x1B[1F\x1B[0J";

/// Partition values by a property.
/// Values with the same property are grouped together in the same Vec.
///
/// E.g.:
///
///   let people_by_age: HashMap<Age, Vec<Person>> = group_by(people, |p|p.age);
///
pub fn group_by<I, F, K, V>(values: I, property: F) -> HashMap<K, Vec<V>>
where
	I: IntoIterator<Item = V>,
	F: Fn(&V) -> K,
	K: std::hash::Hash + Eq,
{
	let mut grouped = HashMap::<K, Vec<V>>::default();
	for value in values.into_iter() {
		grouped.entry(property(&value)).or_default().push(value);
	}
	grouped
}
