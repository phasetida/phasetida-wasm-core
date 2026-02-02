use js_sys::Uint8Array;
use phasetida_core::draw::BufferWithCursor;

pub struct Uint8ArrayWrapper<'a>{
    pub buffer: &'a Uint8Array,
    pub cursor: usize
}

impl<'a> BufferWithCursor for Uint8ArrayWrapper<'a> {
    fn write(&mut self, slice: &[u8]) {
        slice.iter().for_each(|it| {
            self.buffer.set_index(self.cursor as u32, *it);
            self.cursor += 1;
        });
    }
}
