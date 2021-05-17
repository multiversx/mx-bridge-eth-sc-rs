#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi, PartialEq, Debug, Clone)]
pub struct Submission<BigUint: BigUintApi> {
    pub values: Vec<BigUint>,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct Round<BigUint: BigUintApi> {
    pub round_id: u64,
    pub answer: Option<Submission<BigUint>>,
    pub decimals: u8,
    pub description: BoxedBytes,
    pub started_at: u64,
    pub updated_at: u64,
    pub answered_in_round: u64,
}

#[elrond_wasm_derive::contract]
pub trait AggregatorMock {
    #[endpoint(latestRoundData)]
    fn get_latest_round_data(&self) -> OptionalArg<Round<Self::BigUint>> {
        // mock all data as "1"
        let mut submissions = Vec::new();
        for _ in 0..8 {
            submissions.push(Self::BigUint::from(1u32));
        }

        OptionalArg::Some(Round {
            round_id: 0,
            answer: Some(Submission {
                values: submissions,
            }),
            decimals: 0,
            description: BoxedBytes::empty(),
            started_at: 0,
            updated_at: 0,
            answered_in_round: 0,
        })
    }
}
