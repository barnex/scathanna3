use crate::prelude::*;

const NUM_RESAMPLE: usize = 3;

#[derive(Clone, Default, Debug)]
pub(crate) struct Accum {
	partial_sums: [vec3; NUM_RESAMPLE],
	n: [u16; NUM_RESAMPLE],
	cursor: u8,
}

impl Accum {

	pub fn add(&mut self, v: vec3) {
		self.partial_sums[self.cursor as usize] += v;
		self.n[self.cursor as usize] += 1;

		self.cursor += 1;
		if self.cursor >= NUM_RESAMPLE as u8 {
			self.cursor = 0;
		}
	}

	pub fn add_other(&mut self, rhs: &Self) {
		for i in 0..(NUM_RESAMPLE as usize) {
			self.partial_sums[i] += rhs.partial_sums[i];
			self.n[i] += rhs.n[i];
		}
		self.cursor += rhs.cursor;
		if self.cursor >= NUM_RESAMPLE as u8 {
			self.cursor = 0;
		}
	}

	pub fn num_samples(&self) -> u32 {
		self.n.iter().copied().sum::<u16>() as u32
	}

	fn sum(&self) -> vec3 {
		self.partial_sums.iter().copied().sum()
	}

	pub fn avg(&self) -> Option<vec3> {
		if self.num_samples() == 0 {
			None
		} else {
			Some(self.sum() / (self.num_samples() as f32))
		}
	}

	pub fn error(&self) -> Option<f32> {
		let (min, max) = self.confidence_interval()?;
		Some((max.map(linear_to_srgb_f32) - min.map(linear_to_srgb_f32)).len() / 4.0)
	}

	#[inline]
	fn confidence_interval(&self) -> Option<(vec3, vec3)> {
		if self.n.iter().any(|&v| v == 0) {
			return None;
		}

		let partial_avg = self.partial_avg();
		let mut min = vec3::ZERO;
		let mut max = vec3::ZERO;
		for i in 0..3 {
			min[i] = *partial_avg.map(|v| v[i]).iter().min_by(|a, b| partial_max(a, b)).unwrap();
			max[i] = *partial_avg.map(|v| v[i]).iter().max_by(|a, b| partial_max(a, b)).unwrap();
		}

		Some((min, max))
	}

	fn partial_avg(&self) -> [vec3; NUM_RESAMPLE] {
		let mut avg = [vec3::ZERO; NUM_RESAMPLE];
		for i in 0..NUM_RESAMPLE {
			avg[i] = self.partial_sums[i] / self.n[i] as f32
		}
		avg
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn variance_1d() {
		let mut acc = Accum::default();
		for v in [2, 4, 4, 4, 5, 5, 7, 9] {
			acc.add(vec3(v as f32, 0.0, 0.0));
		}
		assert_eq!(acc.num_samples(), 8);
		assert_eq!(acc.avg(), Some(vec3(5.0, 0.0, 0.0)));
	}

	#[test]
	fn variance_3d() {
		let mut acc = Accum::default();
		for v in [2, 4, 4, 4, 5, 5, 7, 9] {
			acc.add(vec3::repeat(v as f32));
		}
		assert_eq!(acc.num_samples(), 8);
		assert_eq!(acc.avg(), Some(vec3(5.0, 5.0, 5.0)));
	}
}
