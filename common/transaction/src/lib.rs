#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use eth_address::EthAddress;

pub mod esdt_safe_batch;
pub mod transaction_status;

// revert protection
pub const MIN_BLOCKS_FOR_FINALITY: u64 = 2;
pub const TX_MULTIRESULT_NR_FIELDS: usize = 6;

pub type TxNonce = u64;
pub type BlockNonce = u64;
pub type SenderAddressRaw<M> = ManagedBuffer<M>;
pub type ReceiverAddressRaw<M> = ManagedBuffer<M>;
pub type TxAsMultiResult<M> = MultiResult6<
    BlockNonce,
    TxNonce,
    SenderAddressRaw<M>,
    ReceiverAddressRaw<M>,
    TokenIdentifier<M>,
    BigUint<M>,
>;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct EthTransaction<M: ManagedTypeApi> {
    pub from: EthAddress<M>,
    pub to: ManagedAddress<M>,
    pub token_id: TokenIdentifier<M>,
    pub amount: BigUint<M>,
    pub tx_nonce: TxNonce,
}

pub type EthTxAsMultiArg<M> =
    MultiArg5<EthAddress<M>, ManagedAddress<M>, TokenIdentifier<M>, BigUint<M>, TxNonce>;

#[derive(NestedEncode, NestedDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct Transaction<M: ManagedTypeApi> {
    pub block_nonce: BlockNonce,
    pub nonce: TxNonce,
    pub from: ManagedBuffer<M>,
    pub to: ManagedBuffer<M>,
    pub token_identifier: TokenIdentifier<M>,
    pub amount: BigUint<M>,
    pub is_refund_tx: bool,
}

impl<M: ManagedTypeApi> From<TxAsMultiResult<M>> for Transaction<M> {
    fn from(tx_as_multiresult: TxAsMultiResult<M>) -> Self {
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
    pub fn into_multiresult(self) -> TxAsMultiResult<M> {
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

// TODO: Remove in next framework version
pub fn managed_address_to_managed_buffer<M: ManagedTypeApi>(
    managed_addr: &ManagedAddress<M>,
) -> ManagedBuffer<M> {
    ManagedBuffer::new_from_bytes(
        elrond_wasm::types::ManagedType::type_manager(managed_addr),
        managed_addr.to_address().as_bytes(),
    )
}

pub fn managed_buffer_to_managed_address<M: ManagedTypeApi>(
    managed_buffer: &ManagedBuffer<M>,
) -> ManagedAddress<M> {
    let mut raw_bytes = [0u8; 32];
    let _ = managed_buffer.load_slice(0, &mut raw_bytes);

    ManagedAddress::new_from_bytes(
        elrond_wasm::types::ManagedType::type_manager(managed_buffer),
        &raw_bytes,
    )
}
