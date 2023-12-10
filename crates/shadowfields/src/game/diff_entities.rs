use super::internal::*;

pub(crate) struct EntityDiff {
	pub players: HashDiff<Player>,
	pub objects: HashDiff<Prop>,
}

impl EntityDiff {
	pub fn diff(old: &Entities, new: &Entities) -> EntityDiff {
		EntityDiff {
			players: HashDiff::diff(&old.players, &new.players),
			objects: HashDiff::diff(&old.props, &new.props),
		}
	}
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct HashDiff<T> {
	insert: HashMap<ID, T>,
	remove: SmallVec<[ID; 2]>,
}

impl<T> HashDiff<T>
where
	T: PartialEq + Clone,
{
	fn diff(old: &HashMap<ID, T>, new: &HashMap<ID, T>) -> HashDiff<T> {
		let mut diff = Self::default();

		for (&id, new_v) in new {
			match old.get(&id) {
				Some(old_v) if old_v == new_v => (),
				_ => {
					diff.insert.insert(id, new_v.clone());
				}
			}
		}

		for &id in old.keys() {
			if !new.contains_key(&id) {
				diff.remove.push(id);
			}
		}

		diff
	}
}

impl<T> Default for HashDiff<T> {
	fn default() -> Self {
		Self { insert: default(), remove: default() }
	}
}
