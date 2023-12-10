use crate::prelude::*;
use std::fmt::Write;

pub(crate) struct Profiler {
	enabled: bool,
	timestep_file: Option<BufWriter<File>>,
	timestamps: [Instant; 4],
	buf: String,
}

#[derive(Clone, Copy)]
enum TS {
	FrameStart = 0,
	Polled = 1,
	Rendered = 2,
	Presented = 3,
}

impl Profiler {
	pub fn new(enabled: bool) -> Self {
		enabled.then(|| LOG.write("Profiling enabled"));
		let now = Instant::now();
		Self {
			enabled,
			timestep_file: enabled.then(|| create("timestep.txt".as_ref()).expect("timestep.txt")),
			timestamps: [now; 4],
			buf: default(),
		}
	}

	pub fn start_new_frame(&mut self, t: Instant, dt: f32) {
		if self.enabled {
			let poll = self.millis_of(TS::Polled);
			let render = self.millis_of(TS::Rendered);
			let present = self.millis_of(TS::Presented);
			let next = (self.timestamps[TS::Presented as usize] - t).as_secs_f32() * 1000.0;

			writeln!(&mut self.buf, "poll: {poll}, render: {render}, present: {present}, to next frame: {next}",).unwrap();
			if self.buf.len() > 1000 {
				print!("{}", self.buf);
				self.buf.clear();
			}

			self.timestamps[TS::FrameStart as usize] = t;
			if let Some(f) = &mut self.timestep_file {
				use std::io::Write;
				writeln!(f, "{dt:.5}").expect("profiler write");
			}
		}
	}

	fn millis_of(&self, what: TS) -> f32 {
		(self.timestamps[what as usize] - self.timestamps[what as usize - 1]).as_secs_f32() * 1000.0
	}

	pub fn gameloop_polled(&mut self) {
		self.timestamps[TS::Polled as usize] = Instant::now();
	}

	pub fn rendered(&mut self) {
		self.timestamps[TS::Rendered as usize] = Instant::now();
	}

	pub fn presented(&mut self) {
		self.timestamps[TS::Presented as usize] = Instant::now();
	}
}
