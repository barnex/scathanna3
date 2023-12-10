use ron::ser::PrettyConfig;

use crate::prelude::*;

pub(crate) fn fmt_bot_overlay(bot: &Bot, state: &Client) -> String {
	let mut buf = String::new();
	use std::fmt::Write as _;

	write!(&mut buf, "{}", fmt_dbg_overlay(state)).unwrap();
	write!(&mut buf, "inputs: {:?}", state.inputs().buttons_pressed).unwrap();
	write!(&mut buf, "{}", ron::ser::to_string_pretty(bot, PrettyConfig::new()).unwrap()).unwrap();
	buf

	//let s =ron::Serializer::new(&mut buf, Some(PrettyConfig::new())).unwrap();
	//String::from_utf8(buf).unwrap()
}
