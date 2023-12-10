use futures::task::Context;
use futures::Future;
use std::{pin::Pin, task::Poll};

use crate::prelude::*;

pub(crate) struct WinitWindow {
	mailbox: WinitMailbox,
	pub inputs: Inputs,
	pub viewport_size: uvec2,
}

impl WinitWindow {
	pub fn new(mailbox: WinitMailbox) -> WinitWindow {
		let viewport_size = match &*mailbox.0.borrow() {
			WinitMailboxInner::RequestRender(_) => panic!("WinitWindow::new called on in-use mailbox"),
			WinitMailboxInner::RequestTick(TickRequest { viewport_size, .. }) => *viewport_size,
		};
		Self {
			mailbox,
			inputs: default(),
			viewport_size,
		}
	}

	pub async fn present_and_wait(&mut self, sg: SceneGraph) {
		let tr = self.mailbox.present_and_wait(sg).await;
		self.inputs = tr.inputs;
		self.viewport_size = tr.viewport_size;
	}
}

#[derive(Clone)]
pub(crate) struct WinitMailbox(Rc<RefCell<WinitMailboxInner>>);

#[derive(Clone)]
pub(crate) enum WinitMailboxInner {
	RequestRender(SceneGraph),
	RequestTick(TickRequest),
}

#[derive(Debug, Clone)]
pub(crate) struct TickRequest {
	pub inputs: Inputs,
	pub viewport_size: uvec2,
}

impl WinitMailbox {
	pub fn new(viewport_size: uvec2) -> Self {
		Self(Rc::new(RefCell::new(WinitMailboxInner::RequestTick(TickRequest { inputs: default(), viewport_size }))))
	}

	pub fn set(&self, msg: WinitMailboxInner) {
		*self.0.borrow_mut() = msg;
	}

	pub fn get(&self) -> WinitMailboxInner {
		self.0.borrow_mut().clone()
	}

	pub fn present_and_wait(&self, sg: SceneGraph) -> impl Future<Output = TickRequest> {
		match &*self.0.borrow() {
			WinitMailboxInner::RequestRender(_) => panic!("present_and_wait: state should be TickRequest"),
			WinitMailboxInner::RequestTick(_) => (),
		};

		self.set(WinitMailboxInner::RequestRender(sg));
		self.clone()
	}
}

impl Future for WinitMailbox {
	type Output = TickRequest;

	fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
		match &*self.as_ref().0.borrow() {
			WinitMailboxInner::RequestRender(_) => Poll::Pending,
			WinitMailboxInner::RequestTick(r) => Poll::Ready(r.clone()),
		}
	}
}
