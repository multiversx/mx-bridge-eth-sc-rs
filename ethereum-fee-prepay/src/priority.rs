elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, TypeAbi)]
pub enum Priority {
    Fast,
    Average,
    Low,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct PriorityGasCosts<BigUint: BigUintApi> {
    pub fast: BigUint,
    pub average: BigUint,
    pub low: BigUint,
}
