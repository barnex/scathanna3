use super::internal::*;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, StreamConfig};
use spatial_filter::*;

pub struct Mixer {
	ringbuf: Arc<Mutex<RingBuffer>>,
}

pub const SAMPLE_RATE: usize = 44100;

impl Mixer {
	pub fn new(size: Duration) -> Result<Self> {
		let samples = (SAMPLE_RATE / 1000) * size.as_millis() as usize;
		let ringbuf = Arc::new(Mutex::new(RingBuffer::new(samples)));

		{
			let ringbuf = ringbuf.clone();
			thread::spawn(|| run(ringbuf).expect("error"));
		}

		Ok(Self { ringbuf })
	}

	const SAMPLING_RATE: f32 = 44100.0;

	pub fn play_raw_stereo_itl(&self, src: impl Iterator<Item = f32>) {
		let mut buf = self.ringbuf.lock().unwrap();
		buf.play_raw_stereo_itl(src)
	}

	pub fn play_raw_mono(&self, src: impl Iterator<Item = f32>) {
		let mut buf = self.ringbuf.lock().unwrap();
		buf.play_raw_mono(src)
	}

	pub fn play_spatial(&self, azimuth: f32, volume: f32, src: impl Iterator<Item = f32> + Clone) {
		self.play_raw_stereo_itl(interleave(duplex_v2(src, Self::SAMPLING_RATE, azimuth, volume)));
	}
}

fn run(buffer: Arc<Mutex<RingBuffer>>) -> Result<()> {
	let device = cpal::default_host().default_output_device().ok_or(anyhow!("No default audio device"))?;

	let config = device.default_output_config()?;
	println!("Output device: {}", device.name()?);
	println!("Default output config: {:?}", config);

	run_generic(&device, &config.into(), buffer)
}

fn run_generic(device: &Device, config: &StreamConfig, buffer: Arc<Mutex<RingBuffer>>) -> Result<()> {
	let stream = device.build_output_stream(
		config,
		move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
			let mut buffer = buffer.lock().unwrap();
			copy_stero_itl(data, &mut || buffer.next_sample())
		},
		move |err| eprintln!("an error occurred on stream: {}", err),
		None,
	)?;
	stream.play()?;

	loop {
		thread::park()
	}
}

fn copy_stero_itl(dst: &mut [f32], next_sample: &mut dyn FnMut() -> (f32, f32)) {
	for i in (0..dst.len()).step_by(2) {
		let sample = next_sample();
		dst[i] = sample.0;
		dst[i + 1] = sample.1;
	}
}
