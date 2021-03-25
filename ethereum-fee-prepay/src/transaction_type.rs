elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, TypeAbi)]
pub enum TransactionType {
    Ethereum, // 21000
    Erc20,
    Erc721,
    Erc1155,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct TransactionGasLimits<BigUint: BigUintApi> {
    pub ethereum: BigUint,
    pub erc20: BigUint,
    pub erc721: BigUint,
    pub erc1155: BigUint,
}
