pub(crate) struct SharedState<'buf> {
    pub buffer: &'buf mut [u8],
    pub offset: usize,
}
