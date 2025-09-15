use crate::SharedState;
use core::{
    cell::RefCell,
    pin::Pin,
    task::{Context, Poll},
};

/// A future that represents the asynchronous writing of a single byte.
///
/// This future is returned by [`Buffer::write_u8`] and will resolve when the byte
/// has been successfully written to the buffer. If the buffer is full, this future
/// will yield (`Poll::Pending`) until space becomes available.
///
/// You typically don't need to interact with this type directly, as it's used
/// internally by the async/await machinery.
pub struct WriteByteFuture<'buf> {
    shared: &'buf RefCell<SharedState<'buf>>,
    value: u8,
}

impl<'buf> WriteByteFuture<'buf> {
    pub(crate) fn new(shared: &'buf RefCell<SharedState<'buf>>, value: u8) -> Self {
        Self { shared, value }
    }
}

impl Future for WriteByteFuture<'_> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let value = self.value;
        let this = self.get_mut();
        let mut shared = this.shared.borrow_mut();
        let off = shared.offset;
        if off < shared.buffer.len() {
            shared.buffer[off] = value;
            shared.offset = off + 1;
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
