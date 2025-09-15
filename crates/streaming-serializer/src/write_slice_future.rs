use crate::SharedState;
use core::{
    cell::RefCell,
    pin::Pin,
    task::{Context, Poll},
};

/// A future that represents the asynchronous writing of a byte slice.
///
/// This future is returned by [`Buffer::write_slice`] and will resolve when all bytes
/// from the source slice have been successfully written to the buffer. If the buffer
/// becomes full during writing, this future will yield (`Poll::Pending`) and resume
/// writing when space becomes available.
///
/// The future tracks its progress internally and can handle writing large slices
/// that span multiple buffer chunks.
///
/// You typically don't need to interact with this type directly, as it's used
/// internally by the async/await machinery.
pub struct WriteSliceFuture<'buf, 'a> {
    shared: &'buf RefCell<SharedState<'buf>>,
    src: &'a [u8],
    written: usize,
}

impl<'buf, 'a> WriteSliceFuture<'buf, 'a> {
    pub(crate) fn new(shared: &'buf RefCell<SharedState<'buf>>, src: &'a [u8]) -> Self {
        Self {
            shared,
            src,
            written: 0,
        }
    }
}

impl Future for WriteSliceFuture<'_, '_> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.written >= this.src.len() {
            return Poll::Ready(());
        }

        let mut shared = this.shared.borrow_mut();
        let capacity_left = shared.buffer.len().saturating_sub(shared.offset);
        if capacity_left == 0 {
            return Poll::Pending;
        }

        let remaining = this.src.len() - this.written;
        let to_write = if remaining < capacity_left {
            remaining
        } else {
            capacity_left
        };

        // SAFETY: slice bounds are ensured by calculations above
        let src_chunk = &this.src[this.written..this.written + to_write];
        let start = shared.offset;
        let end = start + to_write;
        let dst_chunk = &mut shared.buffer[start..end];
        dst_chunk.copy_from_slice(src_chunk);

        this.written += to_write;
        shared.offset += to_write;

        if this.written >= this.src.len() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
