#![no_std]

use fee_estimator_module::GWEI_STRING;
use transaction::{SingleTransferTuple, TransactionStatus};

elrond_wasm::imports!();

#[elrond_wasm::contract]
pub trait MultiTransferEsdt:
    fee_estimator_module::FeeEstimatorModule + token_module::TokenModule
{
    #[init]
    fn init(
        &self,
        fee_estimator_contract_address: ManagedAddress,
        eth_tx_gas_limit: BigUint,
    ) -> SCResult<()> {
        self.fee_estimator_contract_address()
            .set(&fee_estimator_contract_address);

        self.eth_tx_gas_limit().set(&eth_tx_gas_limit);

        // set ticker for "GWEI"
        let gwei_token_id = TokenIdentifier::from(GWEI_STRING);
        self.token_ticker(&gwei_token_id)
            .set(&gwei_token_id.as_managed_buffer());

        Ok(())
    }

    #[only_owner]
    #[endpoint(batchTransferEsdtToken)]
    fn batch_transfer_esdt_token(
        &self,
        #[var_args] transfers: ManagedVarArgs<SingleTransferTuple<Self::Api>>,
    ) -> ManagedMultiResultVec<TransactionStatus> {
        let mut tx_statuses = ManagedVec::new();
        let mut cached_token_ids = ManagedVec::new();
        let mut cached_prices = ManagedVec::new();

        for transfer in transfers {
            let to = &transfer.address;
            let token_id = &transfer.token_id;
            let amount = &transfer.amount;

            if to.is_zero() || self.blockchain().is_smart_contract(to) {
                tx_statuses.push(TransactionStatus::Rejected);
                continue;
            }
            if !self.token_whitelist().contains(token_id)
                || !self.is_local_role_set(token_id, &EsdtLocalRole::Mint)
            {
                tx_statuses.push(TransactionStatus::Rejected);
                continue;
            }

            let queried_fee: BigUint;
            let required_fee = match cached_token_ids.iter().position(|id| &id == token_id) {
                Some(index) => cached_prices.get(index).unwrap_or_else(|| BigUint::zero()),
                None => {
                    queried_fee = self.calculate_required_fee(token_id);
                    cached_token_ids.push(token_id.clone());
                    cached_prices.push(queried_fee.clone());

                    queried_fee
                }
            };

            if amount <= &required_fee {
                tx_statuses.push(TransactionStatus::Rejected);
                continue;
            }

            let amount_to_send = amount - &required_fee;

            self.accumulated_transaction_fees(token_id)
                .update(|fees| *fees += required_fee);

            self.send().esdt_local_mint(token_id, 0, amount);
            self.send().direct(to, token_id, 0, &amount_to_send, &[]);

            tx_statuses.push(TransactionStatus::Executed);
        }

        tx_statuses.into()
    }
}
