use crate::prelude::*;

// pub(crate) fn menu<T: std::fmt::Display>(win: &mut Shell, title: &str, options: &[T]) -> Result<usize> {
// 	use std::fmt::Write;
// 
// 	if options.len() == 0 {
// 		bail!("{title}: no options available")
// 	}
// 
// 	let mut choice: i32 = 0;
// 
// 	let mut buf = String::new();
// 
// 	loop {
// 		let mut sg = win.new_scenegraph();
// 
// 		buf.clear();
// 
// 		if win.inputs.just_pressed(Button::Forward) {
// 			choice -= 1;
// 		}
// 		if win.inputs.just_pressed(Button::Backward) {
// 			choice += 1;
// 		}
// 
// 		if choice < 0 {
// 			choice = options.len() as i32 - 1;
// 		}
// 		if choice >= options.len() as i32 {
// 			choice = 0;
// 		}
// 
// 		if win.inputs.just_pressed(Button::Enter) {
// 			return Ok(choice as usize);
// 		}
// 
// 		let _ = writeln!(&mut buf, "{title}");
// 		for (i, option) in options.iter().enumerate() {
// 			let sel = if i == choice as usize { " => " } else { "    " };
// 			let _ = writeln!(&mut buf, "{sel} {option}");
// 		}
// 
// 		push_text(&mut sg, &buf);
// 
// 		win.present(sg)?;
// 	}
// }
// 