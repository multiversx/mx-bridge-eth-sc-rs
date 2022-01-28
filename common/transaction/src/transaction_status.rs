elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use elrond_wasm::types::ManagedVecItem;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq, Clone, Copy)]
pub enum TransactionStatus {
    None,
    Pending,
    InProgress,
    Executed,
    Rejected,
}

impl From<u8> for TransactionStatus {
    fn from(raw_value: u8) -> Self {
        match raw_value {
            1u8 => Self::Pending,
            2u8 => Self::InProgress,
            3u8 => Self::Executed,
            4u8 => Self::Rejected,
            _ => Self::None,
        }
    }
}

impl TransactionStatus {
    fn as_u8(&self) -> u8 {
        match *self {
            Self::None => 0u8,
            Self::Pending => 1u8,
            Self::InProgress => 2u8,
            Self::Executed => 3u8,
            Self::Rejected => 4u8,
        }
    }
}

impl ManagedVecItem for TransactionStatus {
    const PAYLOAD_SIZE: usize = 1;
    const SKIPS_RESERIALIZATION: bool = true;
    type Ref<'a> = Self;

    fn from_byte_reader<Reader: FnMut(&mut [u8])>(reader: Reader) -> Self {
        u8::from_byte_reader(reader).into()
    }

    fn to_byte_writer<R, Writer: FnMut(&[u8]) -> R>(&self, writer: Writer) -> R {
        <u8 as ManagedVecItem>::to_byte_writer(&self.as_u8(), writer)
    }

    unsafe fn from_byte_reader_as_borrow<'a, Reader: FnMut(&mut [u8])>(
        reader: Reader,
    ) -> Self::Ref<'a> {
        Self::from_byte_reader(reader)
    }
}
