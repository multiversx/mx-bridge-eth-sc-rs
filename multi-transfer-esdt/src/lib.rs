#![no_std]

use transaction::TransactionStatus;

elrond_wasm::imports!();

pub type SingleTransferTuple<BigUint> = (Address, TokenIdentifier, BigUint);

#[elrond_wasm_derive::contract]
pub trait MultiTransferEsdt: fee_estimator_module::FeeEstimatorModule + token_module::TokenModule {
    #[init]
    fn init(
        &self,
        fee_estimator_contract_address: Address,
        eth_tx_gas_limit: Self::BigUint,
        #[var_args] token_whitelist: VarArgs<TokenIdentifier>,
    ) -> SCResult<()> {
        self.fee_estimator_contract_address()
            .set(&fee_estimator_contract_address);

        for token in token_whitelist.into_vec() {
            require!(token.is_valid_esdt_identifier(), "Invalid token ID");
            self.token_whitelist().insert(token);
        }

        self.eth_tx_gas_limit().set(&eth_tx_gas_limit);

        Ok(())
    }

    #[endpoint(batchTransferEsdtToken)]
    fn batch_transfer_esdt_token(
        &self,
        #[var_args] transfers: VarArgs<SingleTransferTuple<Self::BigUint>>,
    ) -> SCResult<MultiResultVec<TransactionStatus>> {
        self.require_caller_owner()?;

        let mut tx_statuses = Vec::new();
        let mut cached_token_ids = Vec::new();
        let mut cached_prices = Vec::new();

        for (i, (to, token_id, amount)) in transfers.into_vec().iter().enumerate() {
            if to.is_zero() || self.blockchain().is_smart_contract(to) {
                tx_statuses.push(TransactionStatus::Rejected);
                continue;
            }
            if !self.token_whitelist().contains(token_id) || !self.is_local_role_set(token_id, &EsdtLocalRole::Mint)
            {
                tx_statuses.push(TransactionStatus::Rejected);
                continue;
            }

            let queried_fee: Self::BigUint;
            let required_fee = match cached_token_ids.iter().position(|&id| id == token_id) {
                Some(index) => &cached_prices[index],
                None => {
                    queried_fee = self.calculate_required_fee(token_id);
                    cached_token_ids.push(token_id);
                    cached_prices.push(queried_fee.clone());

                    &queried_fee
                }
            };

            if required_fee >= amount {
                tx_statuses.push(TransactionStatus::Rejected);
                continue;
            }

            let amount_to_send = amount - required_fee;

            self.accumulated_transaction_fees(token_id)
                .update(|fees| *fees += required_fee);

            self.send().esdt_local_mint(token_id, amount);
            self.send()
                .direct(to, token_id, &amount_to_send, &[i as u8]);

            tx_statuses.push(TransactionStatus::Executed);
        }

        Ok(tx_statuses.into())
    }

    // private

    fn data_or_empty(&self, to: &Address, data: &'static [u8]) -> &[u8] {
        if self.blockchain().is_smart_contract(to) {
            &[]
        } else {
            data
        }
    }
}
