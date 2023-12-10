use rand::Rng;

pub fn pick_random<T>(opts: &[T]) -> Option<&T> {
	match opts.len() {
		0 => None,
		n => Some(&opts[rand::thread_rng().gen_range(0..n)]),
	}
}

pub fn must_pick_random<T: Clone>(opts: &[T]) -> T {
	pick_random(opts).expect("zero options").clone()
}
