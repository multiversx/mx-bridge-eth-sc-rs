use multiversx_sc::codec::{DefaultErrorHandler, NestedEncodeOutput};

use crate::custom_buffer::AlwaysTopEncodedBuffer;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, Clone, ManagedVecItem, Default)]
pub struct CallData<M: ManagedTypeApi> {
    pub endpoint: ManagedBuffer<M>,
    pub gas_limit: u64,
    pub args: ManagedVec<M, ManagedBuffer<M>>,
}

impl<M: ManagedTypeApi> TopEncode for CallData<M> {
    fn top_encode<O>(&self, output: O) -> Result<(), codec::EncodeError>
    where
        O: codec::TopEncodeOutput,
    {
        let mut nested_encode_output = output.start_nested_encode();
        self.dep_encode(&mut nested_encode_output)
    }
}

impl<M: ManagedTypeApi> NestedEncode for CallData<M> {
    fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), codec::EncodeError> {
        let endpoint_len_u8 = self.endpoint.len() as u8;
        dest.push_byte(endpoint_len_u8);

        let endpoint_buffer = AlwaysTopEncodedBuffer::new(self.endpoint.clone());
        endpoint_buffer.dep_encode(dest)?;

        let gas_limit_u32 = self.gas_limit as u32;
        gas_limit_u32.dep_encode(dest)?;

        let args_len_u8 = self.args.len() as u8;
        dest.push_byte(args_len_u8);

        for arg in &self.args {
            arg.dep_encode(dest)?;
        }

        Result::Ok(())
    }
}

impl<M: ManagedTypeApi> NestedDecode for CallData<M> {
    fn dep_decode<I: codec::NestedDecodeInput>(input: &mut I) -> Result<Self, DecodeError> {
        let mut temp_buffer = [0u8; u8::MAX as usize + 1];
        let endpoint_len_u8 = u8::dep_decode(input)?;
        input.read_into(
            &mut temp_buffer[0..endpoint_len_u8 as usize],
            DefaultErrorHandler,
        )?;
        let endpoint_name =
            ManagedBuffer::new_from_bytes(&temp_buffer[0..endpoint_len_u8 as usize]);

        let gas_limit_u32 = u32::dep_decode(input)?;
        let gas_limit = gas_limit_u32 as u64;

        let args_len_u8 = u8::dep_decode(input)?;
        let mut args = ManagedVec::new();
        for _ in 0..args_len_u8 {
            let arg = ManagedBuffer::dep_decode(input)?;
            args.push(arg);
        }

        if !input.is_depleted() {
            return core::result::Result::Err(DecodeError::from("Input too long"));
        }

        core::result::Result::Ok(Self {
            endpoint: endpoint_name,
            gas_limit,
            args,
        })
    }
}
