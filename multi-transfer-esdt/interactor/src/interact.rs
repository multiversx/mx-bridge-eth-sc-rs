#![allow(non_snake_case)]

mod config;
mod proxy;

pub use config::Config;
use multiversx_sc_snippets::imports::*;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};
use transaction::EthTransaction;

const STATE_FILE: &str = "state.toml";

pub async fn multi_transfer_esdt_cli() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();
    let cmd = args.next().expect("at least one argument required");
    let mut interact = ContractInteract::new(Config::new()).await;
    match cmd.as_str() {
        // "deploy" => interact.deploy().await,
        "upgrade" => interact.upgrade().await,
        "batchTransferEsdtToken" => interact.batch_transfer_esdt_token().await,
        "myStorage" => interact.my_storage().await,
        "moveRefundBatchToSafe" => interact.move_refund_batch_to_safe().await,
        "addUnprocessedRefundTxToBatch" => interact.add_unprocessed_refund_tx_to_batch().await,
        // "myDistributePayments" => interact.my_distribute_payments().await,
        "setMaxTxBatchSize" => interact.set_max_tx_batch_size().await,
        "setMaxTxBatchBlockDuration" => interact.set_max_tx_batch_block_duration().await,
        "getCurrentTxBatch" => interact.get_current_tx_batch().await,
        "getFirstBatchAnyStatus" => interact.get_first_batch_any_status().await,
        "getBatch" => interact.get_batch().await,
        "getBatchStatus" => interact.get_batch_status().await,
        "getFirstBatchId" => interact.first_batch_id().await,
        "getLastBatchId" => interact.last_batch_id().await,
        "setMaxBridgedAmount" => interact.set_max_bridged_amount().await,
        "getMaxBridgedAmount" => interact.max_bridged_amount().await,
        _ => panic!("unknown command: {}", &cmd),
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    contract_address: Option<Bech32Address>,
}

impl State {
    // Deserializes state from file
    pub fn load_state() -> Self {
        if Path::new(STATE_FILE).exists() {
            let mut file = std::fs::File::open(STATE_FILE).unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            toml::from_str(&content).unwrap()
        } else {
            Self::default()
        }
    }

    /// Sets the contract address
    pub fn set_address(&mut self, address: Bech32Address) {
        self.contract_address = Some(address);
    }

    /// Returns the contract address
    pub fn current_address(&self) -> &Bech32Address {
        self.contract_address
            .as_ref()
            .expect("no known contract, deploy first")
    }
}

impl Drop for State {
    // Serializes state to file
    fn drop(&mut self) {
        let mut file = std::fs::File::create(STATE_FILE).unwrap();
        file.write_all(toml::to_string(self).unwrap().as_bytes())
            .unwrap();
    }
}

pub struct ContractInteract {
    pub interactor: Interactor,
    pub wallet_address: Address,
    contract_code: BytesValue,
    state: State,
}

impl ContractInteract {
    pub async fn new(config: Config) -> Self {
        // let config = Config::new();
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator());

        interactor.set_current_dir_from_workspace("multi-transfer-esdt");
        let wallet_address = interactor.register_wallet(test_wallets::alice()).await;

        // Useful in the chain simulator setting
        // generate blocks until ESDTSystemSCAddress is enabled
        interactor.generate_blocks_until_epoch(1).await.unwrap();

        let contract_code = BytesValue::interpret_from(
            "mxsc:../output/multi-transfer-esdt.mxsc.json",
            &InterpreterContext::default(),
        );

        ContractInteract {
            interactor,
            wallet_address,
            contract_code,
            state: State::load_state(),
        }
    }

    pub async fn deploy(&mut self) -> Bech32Address {
        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(100_000_000u64)
            .typed(proxy::MultiTransferEsdtProxy)
            .init()
            .code(&self.contract_code)
            .returns(ReturnsNewAddress)
            .run()
            .await;
        let new_address_bech32 = bech32::encode(&new_address);
        self.state.set_address(Bech32Address::from_bech32_string(
            new_address_bech32.clone(),
        ));

        println!("new address: {new_address_bech32}");

        new_address.into()
    }

    pub async fn upgrade(&mut self) {
        let response = self
            .interactor
            .tx()
            .to(self.state.current_address())
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(proxy::MultiTransferEsdtProxy)
            .upgrade()
            .code(&self.contract_code)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsNewAddress)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn batch_transfer_esdt_token(&mut self) {
        let batch_id = 0u64;
        let transfers = MultiValueVec::<EthTransaction<StaticApi>>::new();

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::MultiTransferEsdtProxy)
            .batch_transfer_esdt_token(batch_id, transfers)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn my_storage(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::MultiTransferEsdtProxy)
            .my_storage()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn move_refund_batch_to_safe(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::MultiTransferEsdtProxy)
            .move_refund_batch_to_safe()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn add_unprocessed_refund_tx_to_batch(&mut self) {
        let tx_id = 0u64;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::MultiTransferEsdtProxy)
            .add_unprocessed_refund_tx_to_batch(tx_id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn my_distribute_payments(&mut self, to: Bech32Address) {
        // let to = bech32::decode("");
        let payments = ManagedVec::from_single_item(EsdtTokenPayment::new(
            TokenIdentifier::from_esdt_bytes(b"WEGLD-a28c59"),
            0u64,
            BigUint::from(10000000000000000u64),
        ));

        // payments.push(EsdtTokenPayment::new(
        //     TokenIdentifier::from_esdt_bytes(b"ONE-83a7c0"),
        //     0u64,
        //     BigUint::from(100000000000000000u64),
        // ));

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(600_000_000u64)
            .typed(proxy::MultiTransferEsdtProxy)
            .my_distribute_payments(to, payments)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn set_max_tx_batch_size(&mut self) {
        let new_max_tx_batch_size = 0u32;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::MultiTransferEsdtProxy)
            .set_max_tx_batch_size(new_max_tx_batch_size)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn set_max_tx_batch_block_duration(&mut self) {
        let new_max_tx_batch_block_duration = 0u64;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::MultiTransferEsdtProxy)
            .set_max_tx_batch_block_duration(new_max_tx_batch_block_duration)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn get_current_tx_batch(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::MultiTransferEsdtProxy)
            .get_current_tx_batch()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn get_first_batch_any_status(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::MultiTransferEsdtProxy)
            .get_first_batch_any_status()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn get_batch(&mut self) {
        let batch_id = 0u64;

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::MultiTransferEsdtProxy)
            .get_batch(batch_id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn get_batch_status(&mut self) {
        let batch_id = 0u64;

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::MultiTransferEsdtProxy)
            .get_batch_status(batch_id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn first_batch_id(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::MultiTransferEsdtProxy)
            .first_batch_id()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn last_batch_id(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::MultiTransferEsdtProxy)
            .last_batch_id()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn set_max_bridged_amount(&mut self) {
        let token_id = TokenIdentifier::from_esdt_bytes(&b""[..]);
        let max_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::MultiTransferEsdtProxy)
            .set_max_bridged_amount(token_id, max_amount)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn max_bridged_amount(&mut self) {
        let token_id = TokenIdentifier::from_esdt_bytes(&b""[..]);

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::MultiTransferEsdtProxy)
            .max_bridged_amount(token_id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn my_transfer(&mut self, to: Bech32Address) {
        let token_id = TokenIdentifier::from_esdt_bytes(b"WEGLD-a28c59");

        let result_value = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(to)
            .single_esdt(&token_id, 0, &BigUint::from(100000000000000000u64))
            .run()
            .await;

        println!("Result: {result_value:?}");
    }
}
