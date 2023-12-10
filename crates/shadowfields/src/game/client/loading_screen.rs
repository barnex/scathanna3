use crate::prelude::*;

/// Run `f` in a background thread until complete,
/// show the logs scrolling by the meantime.
pub(crate) async fn with_loading_screen<T, F>(win: &mut WinitWindow, f: F) -> T
where
	F: FnOnce() -> T + Send + 'static,
	T: Send + 'static,
{
	let h = thread::spawn(f);
	while !h.is_finished() {
		let mut sg = SceneGraph::new(win.viewport_size);
		layout_log(&mut sg);
		win.present_and_wait(sg).await;
	}
	h.join().expect("child thread panic")
}

fn layout_log(sg: &mut SceneGraph) {
	let ctx = ctx();
	let max_lines = viewport_size_chars(sg.viewport_size).y();
	let tail = LOG.tail(max_lines as usize);
	sg.push(Object::new(Arc::new(ctx.upload_meshbuffer(&layout_text_bottom(sg.viewport_size, &tail))), ctx.shader_pack.text()));
}
