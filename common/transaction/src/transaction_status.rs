elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(
    TopEncode,
    TopDecode,
    NestedEncode,
    NestedDecode,
    TypeAbi,
    PartialEq,
    Clone,
    Copy,
    ManagedVecItem,
)]
pub enum TransactionStatus {
    None,
    Pending,
    InProgress,
    Executed,
    Rejected,
}
