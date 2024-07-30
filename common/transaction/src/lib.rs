#![no_std]

use codec::NestedDecodeInput;
use multiversx_sc::derive_imports::*;
use multiversx_sc::imports::*;

use eth_address::EthAddress;
pub mod transaction_status;

// revert protection
pub const MIN_BLOCKS_FOR_FINALITY: u64 = 10;
pub const TX_MULTIRESULT_NR_FIELDS: usize = 6;

pub type TxNonce = u64;
pub type BlockNonce = u64;
pub type SenderAddressRaw<M> = ManagedBuffer<M>;
pub type ReceiverAddressRaw<M> = ManagedBuffer<M>;
pub type TxAsMultiValue<M> = MultiValue6<
    BlockNonce,
    TxNonce,
    SenderAddressRaw<M>,
    ReceiverAddressRaw<M>,
    TokenIdentifier<M>,
    BigUint<M>,
>;
pub type PaymentsVec<M> = ManagedVec<M, EsdtTokenPayment<M>>;
pub type TxBatchSplitInFields<M> = MultiValue2<u64, MultiValueEncoded<M, TxAsMultiValue<M>>>;

#[type_abi]
#[derive(NestedEncode, TopDecode, TopEncode, Clone, ManagedVecItem)]
pub struct CallData<M: ManagedTypeApi> {
    pub endpoint: ManagedBuffer<M>,
    pub gas_limit: u64,
    pub args: ManagedOption<M, ManagedVec<M, ManagedBuffer<M>>>,
}

impl<M: ManagedTypeApi> Default for CallData<M> {
    fn default() -> Self {
        CallData {
            endpoint: ManagedBuffer::new(),
            gas_limit: 0u64,
            args: ManagedOption::none(),
        }
    }
}

impl<M: ManagedTypeApi> NestedDecode for CallData<M> {
    fn dep_decode_or_handle_err<I, H>(nested_buffer: &mut I, h: H) -> Result<Self, H::HandledErr>
    where
        I: codec::NestedDecodeInput,
        H: codec::DecodeErrorHandler,
    {
        let mut endpoint = ManagedBuffer::new();
        if !nested_buffer.is_depleted() {
            endpoint = ManagedBuffer::dep_decode_or_handle_err(nested_buffer, h)?;
        }

        let mut gas_limit = 0u64;
        if !nested_buffer.is_depleted() {
            gas_limit = u64::dep_decode_or_handle_err(nested_buffer, h)?;
        }

        let mut args: ManagedVec<M, ManagedBuffer<M>> = ManagedVec::new();
        while !nested_buffer.is_depleted() {
            args.push(ManagedBuffer::dep_decode_or_handle_err(nested_buffer, h)?);
        }

        let args: ManagedOption<M, ManagedVec<M, ManagedBuffer<M>>> = if args.is_empty() {
            ManagedOption::none()
        } else {
            ManagedOption::some(args)
        };

        Result::Ok(CallData {
            endpoint,
            gas_limit,
            args,
        })
    }
}

#[type_abi]
#[derive(NestedEncode, NestedDecode, TopEncode, Clone, ManagedVecItem)]
pub struct EthTransaction<M: ManagedTypeApi> {
    pub from: EthAddress<M>,
    pub to: ManagedAddress<M>,
    pub token_id: TokenIdentifier<M>,
    pub amount: BigUint<M>,
    pub tx_nonce: TxNonce,
    pub call_data: CallData<M>,
}

impl<M: ManagedTypeApi> TopDecode for EthTransaction<M> {
    fn top_decode_or_handle_err<I, H>(input: I, h: H) -> Result<Self, H::HandledErr>
    where
        I: codec::TopDecodeInput,
        H: codec::DecodeErrorHandler,
    {
        let mut nested_buffer = input.into_nested_buffer();
        let from = EthAddress::dep_decode_or_handle_err(&mut nested_buffer, h)?;
        let to = ManagedAddress::dep_decode_or_handle_err(&mut nested_buffer, h)?;
        let token_id = TokenIdentifier::dep_decode_or_handle_err(&mut nested_buffer, h)?;
        let amount = BigUint::dep_decode_or_handle_err(&mut nested_buffer, h)?;
        let tx_nonce = TxNonce::dep_decode_or_handle_err(&mut nested_buffer, h)?;

        let mut call_data = CallData::default();

        if !nested_buffer.is_depleted() {
            call_data = CallData::dep_decode_or_handle_err(&mut nested_buffer, h)?;
        }

        Result::Ok(EthTransaction {
            from,
            to,
            token_id,
            amount,
            tx_nonce,
            call_data,
        })
    }
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem, Clone)]
pub struct Transaction<M: ManagedTypeApi> {
    pub block_nonce: BlockNonce,
    pub nonce: TxNonce,
    pub from: ManagedBuffer<M>,
    pub to: ManagedBuffer<M>,
    pub token_identifier: TokenIdentifier<M>,
    pub amount: BigUint<M>,
    pub is_refund_tx: bool,
}

impl<M: ManagedTypeApi> From<TxAsMultiValue<M>> for Transaction<M> {
    fn from(tx_as_multiresult: TxAsMultiValue<M>) -> Self {
        let (block_nonce, nonce, from, to, token_identifier, amount) =
            tx_as_multiresult.into_tuple();

        Transaction {
            block_nonce,
            nonce,
            from,
            to,
            token_identifier,
            amount,
            is_refund_tx: false,
        }
    }
}

impl<M: ManagedTypeApi> Transaction<M> {
    pub fn into_multiresult(self) -> TxAsMultiValue<M> {
        (
            self.block_nonce,
            self.nonce,
            self.from,
            self.to,
            self.token_identifier,
            self.amount,
        )
            .into()
    }
}
