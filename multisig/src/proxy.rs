use transaction::*;

elrond_wasm::imports!();

use transaction::{Priority, TransactionType};

#[elrond_wasm_derive::callable(EgldEsdtSwapProxy)]
pub trait EgldEsdtSwap {
    fn issueWrappedEgld(
        &self,
        token_display_name: BoxedBytes,
        token_ticker: BoxedBytes,
    ) -> ContractCall<BigUint, ()>; // payable EGLD
    fn setLocalMintRole(&self) -> ContractCall<BigUint, ()>;
}

#[elrond_wasm_derive::callable(EsdtSafeProxy)]
pub trait EsdtSafe {
    fn addTokenToWhitelist(&self, token_id: TokenIdentifier) -> ContractCall<BigUint, ()>;
    fn removeTokenFromWhitelist(&self, token_id: TokenIdentifier) -> ContractCall<BigUint, ()>;
    fn getNextPendingTransaction(
        &self,
    ) -> ContractCall<
        BigUint,
        OptionalResult<MultiResult5<Nonce, Address, Address, TokenIdentifier, BigUint>>,
    >;
    fn setTransactionStatus(
        &self,
        sender: Address,
        nonce: Nonce,
        transaction_status: TransactionStatus,
    ) -> ContractCall<BigUint, ()>;
}

#[elrond_wasm_derive::callable(MultiTransferEsdtProxy)]
pub trait MultiTransferEsdt {
    fn issueEsdtToken(
        &self,
        token_display_name: BoxedBytes,
        token_ticker: BoxedBytes,
    ) -> ContractCall<BigUint, ()>; // payable EGLD
    fn setLocalMintRole(&self, token_id: TokenIdentifier) -> ContractCall<BigUint, ()>;
    fn transferEsdtToken(
        &self,
        to: Address,
        token_id: TokenIdentifier,
        amount: BigUint,
    ) -> ContractCall<BigUint, ()>;
}

#[elrond_wasm_derive::callable(EthereumFeePrepayProxy)]
pub trait EthereumFeePrepay {
    fn payFee(
        &self,
        address: &Address,
        relayer: &Address,
        transaction_type: TransactionType,
        priority: Priority,
    ) -> ContractCall<BigUint, ()>;
    fn addToWhitelist(&self, address: &Address) -> ContractCall<BigUint, ()>;
}
