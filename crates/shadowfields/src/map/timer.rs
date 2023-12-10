use crate::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Timer {
	remaining: f32,
	just_finished: bool,
	pub duration: f32,
	repeating: bool,
}

impl Default for Timer {
	fn default() -> Self {
		Self::one_off_ready(0.0)
	}
}

impl Timer {
	// A one-off timer (fires once, does not repeat until reset).
	pub fn one_off(duration: f32) -> Self {
		Self {
			duration,
			just_finished: false,
			remaining: duration,
			repeating: false,
		}
	}

	pub fn unarmed() -> Self {
		Self {
			duration: 0.0,
			just_finished: false,
			remaining: -1.0,
			repeating: false,
		}
	}

	// A one-off timer (fires once, does not repeat until reset).
	// Starts in the `just_finished` state.
	pub fn one_off_ready(duration: f32) -> Self {
		Self {
			duration,
			just_finished: false,
			remaining: 0.0,
			repeating: false,
		}
	}

	pub fn repeating(duration: f32) -> Self {
		Self {
			duration,
			just_finished: false,
			remaining: duration,
			repeating: true,
		}
	}

	pub fn tick(&mut self, dt: f32) -> &mut Self {
		self.just_finished = false;
		if self.remaining >= 0.0 {
			self.remaining -= dt;
			if self.remaining < 0.0 {
				self.just_finished = true;
				if self.repeating {
					self.remaining += self.duration;
				}
			}
		}
		self
	}

	pub fn is_running(&self) -> bool {
		self.remaining >= 0.0
	}

	pub fn is_idle(&self) -> bool {
		!self.is_running()
	}

	pub fn ensure_running(&mut self) -> &mut Self {
		if !self.is_running() {
			self.reset();
		}
		self
	}

	pub fn reset(&mut self) -> &mut Self {
		self.remaining = self.duration;
		self.just_finished = false;
		self
	}

	pub fn set_if_idle(&mut self, duration: f32) -> &mut Self {
		if !self.is_running() {
			self.duration = duration;
			self.reset();
		}
		self
	}

	pub fn set(&mut self, duration: f32) -> &mut Self {
		self.duration = duration;
		self.reset();
		self
	}

	pub fn just_finished(&self) -> bool {
		self.just_finished
	}
}
