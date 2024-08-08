#![no_std]

#[multiversx_sc::contract]
pub trait TestCallerContract {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    #[payable("*")]
    #[endpoint(callPayable)]
    fn call_payable(&self) {}

    #[endpoint(callNonPayable)]
    fn call_non_payable(&self) {}
}
