multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub struct AlwaysTopEncodedBuffer<M: ManagedTypeApi> {
    buffer: ManagedBuffer<M>,
}

impl<M: ManagedTypeApi> AlwaysTopEncodedBuffer<M> {
    #[inline(always)]
    pub fn new(buffer: ManagedBuffer<M>) -> Self {
        Self { buffer }
    }
}

impl<M: ManagedTypeApi> TopEncode for AlwaysTopEncodedBuffer<M> {
    fn top_encode<O>(&self, output: O) -> Result<(), codec::EncodeError>
    where
        O: codec::TopEncodeOutput,
    {
        self.buffer.top_encode(output)
    }
}

impl<M: ManagedTypeApi> NestedEncode for AlwaysTopEncodedBuffer<M> {
    fn dep_encode<O: codec::NestedEncodeOutput>(
        &self,
        dest: &mut O,
    ) -> Result<(), codec::EncodeError> {
        let mut output_buffer = ManagedBuffer::<M>::new();
        self.buffer.top_encode(&mut output_buffer)?;

        let mut output_buffer_bytes = [0u8; u8::MAX as usize + 1];
        let slice_ref = output_buffer.load_to_byte_array(&mut output_buffer_bytes);
        dest.write(slice_ref);

        Result::Ok(())
    }
}
