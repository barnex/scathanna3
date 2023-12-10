use crate::settings::Settings;
use crate::shell::Shell3;

pub fn play_main(settings: Settings) -> anyhow::Result<()> {
	Shell3::run(settings)
}
