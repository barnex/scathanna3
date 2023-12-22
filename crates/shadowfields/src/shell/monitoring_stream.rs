use crate::prelude::*;

/// Stream N columns of monitoring data to a text file, non-blocking.
pub struct MonitoringStream<const N: usize> {
	send: SyncSender<[f32; N]>,
	_worker: Handle, // here so it gets dropped together
}

impl<const N: usize> MonitoringStream<N> {
	/// Create stream to given file. E.g. "player_position.txt".
	/// Stream will flush upon drop and log errors, if any.
	pub fn new(fname: &str) -> Result<Self> {
		log::info!("streaming monitoring data to {fname}");
		let (send, recv) = mpsc::sync_channel(32 * 1024);
		let mut file = create(fname.as_ref())?;
		let fname = PathBuf::from(fname);

		let _worker = Handle(Some(thread::spawn(move || {
			for item in recv {
				for v in item {
					write!(&mut file, "{v} ")?;
				}
				writeln!(&mut file, "")?;
			}
			log::info!("closing monitoring stream");
			file.flush().with_context(|| anyhow!("monitoring stream: close {fname:?}"))
		})));

		Ok(Self { send, _worker })
	}

	/// Stream one data point to file.
	/// Will be formatted as a line of space-separated text values.
	pub fn stream(&mut self, v: [f32; N]) {
		let _ = self.send.send(v).context("monitoring: send data").log_err();
	}
}

/// Upon drop, wait for worker thread to finish (flush file).
struct Handle(Option<thread::JoinHandle<Result<()>>>);

impl Drop for Handle {
	fn drop(&mut self) {
		if let Some(handle) = self.0.take() {
			log::info!("dropping monitoring stream");
			match handle.join() {
				Ok(Ok(())) => log::info!("monitoring stream closed"),
				Ok(Err(e)) => log::error!("monitoring stream: {e:#?}"),
				Err(e) => log::error!("monitoring stream: panic: {e:#?}"),
			}
		}
	}
}
