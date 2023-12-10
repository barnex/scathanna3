//!
//! Client console for entering commands. E.g. "summon shield", "switch my_map".
//!

use super::internal::*;

pub(crate) async fn console_system(state: &mut Client) -> Result<()> {
	if state.inputs().just_pressed(Button::Console) {
		if let Some(cmd) = enter_text_input(state).await {
			match exec_command(state, &cmd) {
				Ok(()) => LOG.write("OK"),
				Err(e) => LOG.write(format!("{e:#}")),
			}
		}
	}
	Ok(())
}
/// Go into console mode until the user provided input (potentially empty).
pub(crate) async fn enter_text_input(state: &mut Client) -> Option<String> {
	let mut cmd = String::new();

	loop {
		for chr in state.win.inputs.received_characters().chars() {
			match chr {
				'\x08' | '\x7f' => drop(cmd.pop()), // backspace (linux, windows | mac)
				'\r' => return Some(cmd.trim().into()),
				chr => {
					if !chr.is_ascii_control() {
						cmd.push(chr)
					}
				}
			}
		}

		let mut sg = SceneGraph::new(state.win.viewport_size);
		draw_cli(&mut sg, &cmd);

		state.win.present_and_wait(sg).await;

		if state.win.inputs.just_pressed(Button::Esc) || state.win.inputs.just_pressed(Button::Console) {
			return None;
		}
	}
}

fn draw_cli(sg: &mut SceneGraph, cmd: &str) {
	let ctx = ctx();
	let viewport_size = sg.viewport_size;

	let cli_text = Object::new(Arc::new(ctx.upload_meshbuffer(&layout_text_bottom(viewport_size, &format!(">{cmd}")))), ctx.shader_pack.text());
	sg.objects.push(cli_text);

	// show log when typing in the CLI
	let log_text = LOG.tail(viewport_size_chars(sg.viewport_size).y() as usize) + "\n";
	let log_text = Object::new(Arc::new(ctx.upload_meshbuffer(&layout_text_bottom(viewport_size, &log_text))), ctx.shader_pack.text());
	sg.objects.push(log_text);
}
