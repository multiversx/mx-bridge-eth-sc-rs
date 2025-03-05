#[multiversx_sc::module]
pub trait EventsModule {
    #[event("executeSuccesfullyFinished")]
    fn execute_succesfully_finished(&self, #[indexed] tx_id: usize);

    #[event("executeGeneratedRefund")]
    fn execute_generated_refund(&self, #[indexed] tx_id: usize);
}
