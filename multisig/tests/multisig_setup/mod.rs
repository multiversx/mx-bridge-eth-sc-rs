use esdt_safe::EsdtSafe;
use multi_transfer_esdt::MultiTransferEsdt;
use multisig::Multisig;
use multiversx_price_aggregator_sc::PriceAggregator;
use multiversx_sc::{
    codec::multi_types::OptionalValue,
    types::{Address, EsdtLocalRole, MultiValueEncoded},
};
use multiversx_sc_modules::{pause::PauseModule, staking::StakingModule};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_buffer, managed_egld_token_id, managed_token_id,
    rust_biguint,
    whitebox_legacy::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};
use token_module::TokenModule;
use tx_batch_module::TxBatchModule;

pub static WEGLD_TOKEN_ID: &[u8] = b"WEGLD-123456";
pub static ETH_TOKEN_ID: &[u8] = b"ETH-123456";
pub static GWEI_TOKEN_ID: &[u8] = b"GWEI";
pub static BRIDGE_TOKEN_ID: &[u8] = b"BRIDGE";

pub const ETH_TX_GAS_LIMIT: u64 = 150_000;
pub const STAKE_AMOUNT: u64 = 1_000;

pub struct MultisigSetup<
    PriceAggregatorBuilder,
    EsdtSafeBuilder,
    MultiTransferBuilder,
    MultisigBuilder,
> where
    PriceAggregatorBuilder:
        'static + Copy + Fn() -> multiversx_price_aggregator_sc::ContractObj<DebugApi>,
    EsdtSafeBuilder: 'static + Copy + Fn() -> esdt_safe::ContractObj<DebugApi>,
    MultiTransferBuilder: 'static + Copy + Fn() -> multi_transfer_esdt::ContractObj<DebugApi>,
    MultisigBuilder: 'static + Copy + Fn() -> multisig::ContractObj<DebugApi>,
{
    pub b_mock: BlockchainStateWrapper,
    pub owner_addr: Address,
    pub price_agg_wrapper: ContractObjWrapper<
        multiversx_price_aggregator_sc::ContractObj<DebugApi>,
        PriceAggregatorBuilder,
    >,
    pub esdt_safe_wrapper: ContractObjWrapper<esdt_safe::ContractObj<DebugApi>, EsdtSafeBuilder>,
    pub multi_transfer_wrapper:
        ContractObjWrapper<multi_transfer_esdt::ContractObj<DebugApi>, MultiTransferBuilder>,
    pub multisig_wrapper: ContractObjWrapper<multisig::ContractObj<DebugApi>, MultisigBuilder>,
}

impl<PriceAggregatorBuilder, EsdtSafeBuilder, MultiTransferBuilder, MultisigBuilder>
    MultisigSetup<PriceAggregatorBuilder, EsdtSafeBuilder, MultiTransferBuilder, MultisigBuilder>
where
    PriceAggregatorBuilder:
        'static + Copy + Fn() -> multiversx_price_aggregator_sc::ContractObj<DebugApi>,
    EsdtSafeBuilder: 'static + Copy + Fn() -> esdt_safe::ContractObj<DebugApi>,
    MultiTransferBuilder: 'static + Copy + Fn() -> multi_transfer_esdt::ContractObj<DebugApi>,
    MultisigBuilder: 'static + Copy + Fn() -> multisig::ContractObj<DebugApi>,
{
    pub fn new(
        price_aggregator_builder: PriceAggregatorBuilder,
        esdt_safe_builder: EsdtSafeBuilder,
        multi_transfer_builder: MultiTransferBuilder,
        multisig_builder: MultisigBuilder,
    ) -> Self {
        let rust_zero = rust_biguint!(0);
        let mut b_mock = BlockchainStateWrapper::new();
        let owner_addr = b_mock.create_user_account(&rust_zero);

        let mut oracles = Vec::with_capacity(5);
        for _ in 0..5 {
            let oracle = b_mock.create_user_account(&rust_biguint!(100));
            oracles.push(oracle);
        }

        // Price Aggregator setup

        let price_agg_wrapper = b_mock.create_sc_account(
            &rust_zero,
            Some(&owner_addr),
            price_aggregator_builder,
            "price aggregator wasm path",
        );
        b_mock
            .execute_tx(&owner_addr, &price_agg_wrapper, &rust_zero, |sc| {
                let mut oracles_managed = MultiValueEncoded::new();
                for oracle in &oracles {
                    oracles_managed.push(managed_address!(oracle));
                }

                sc.init(
                    managed_egld_token_id!(),
                    managed_biguint!(20),
                    managed_biguint!(10),
                    3,
                    3,
                    oracles_managed,
                );

                sc.set_pair_decimals(
                    managed_buffer!(GWEI_TOKEN_ID),
                    managed_buffer!(BRIDGE_TOKEN_ID),
                    6,
                );
                sc.set_pair_decimals(
                    managed_buffer!(GWEI_TOKEN_ID),
                    managed_buffer!(WEGLD_TOKEN_ID),
                    6,
                );

                sc.unpause_endpoint();
            })
            .assert_ok();

        for i in 0..2 {
            b_mock
                .execute_tx(&oracles[i], &price_agg_wrapper, &rust_biguint!(100), |sc| {
                    sc.stake();
                })
                .assert_ok();
        }

        for i in 0..2 {
            b_mock
                .execute_tx(&oracles[i], &price_agg_wrapper, &rust_zero, |sc| {
                    sc.submit(
                        managed_buffer!(GWEI_TOKEN_ID),
                        managed_buffer!(BRIDGE_TOKEN_ID),
                        0,
                        managed_biguint!(10),
                        6,
                    );
                })
                .assert_ok();
        }

        b_mock
            .execute_tx(&oracles[0], &price_agg_wrapper, &rust_zero, |sc| {
                sc.submit(
                    managed_buffer!(GWEI_TOKEN_ID),
                    managed_buffer!(BRIDGE_TOKEN_ID),
                    0,
                    managed_biguint!(1),
                    6,
                );
            })
            .assert_ok();

        b_mock
            .execute_tx(&oracles[0], &price_agg_wrapper, &rust_zero, |sc| {
                sc.submit(
                    managed_buffer!(GWEI_TOKEN_ID),
                    managed_buffer!(WEGLD_TOKEN_ID),
                    0,
                    managed_biguint!(10),
                    6,
                );
            })
            .assert_ok();

        // Create multisig wrapper first - needed for the other contracts

        let multisig_wrapper = b_mock.create_sc_account(
            &rust_zero,
            Some(&owner_addr),
            multisig_builder,
            "multisig wasm path",
        );

        // Esdt Safe setup

        let esdt_safe_wrapper = b_mock.create_sc_account(
            &rust_zero,
            Some(multisig_wrapper.address_ref()),
            esdt_safe_builder,
            "esdt safe wasm path",
        );

        let multi_transfer_wrapper = b_mock.create_sc_account(
            &rust_zero,
            Some(multisig_wrapper.address_ref()),
            multi_transfer_builder,
            "multi-transfer wasm path",
        );

        b_mock
            .execute_tx(
                multisig_wrapper.address_ref(),
                &esdt_safe_wrapper,
                &rust_zero,
                |sc| {
                    sc.init(
                        managed_address!(price_agg_wrapper.address_ref()),
                        managed_biguint!(ETH_TX_GAS_LIMIT),
                    );

                    sc.set_multi_transfer_contract_address(OptionalValue::Some(managed_address!(
                        multi_transfer_wrapper.address_ref()
                    )));

                    sc.add_token_to_whitelist(
                        managed_token_id!(WEGLD_TOKEN_ID),
                        managed_buffer!(b"WEGLD"),
                        true,
                        OptionalValue::Some(managed_biguint!(500_000)),
                    );
                    sc.add_token_to_whitelist(
                        managed_token_id!(ETH_TOKEN_ID),
                        managed_buffer!(b"ETH"),
                        true,
                        OptionalValue::Some(managed_biguint!(500_000)),
                    );

                    sc.set_accumulated_burned_tokens(
                        managed_token_id!(WEGLD_TOKEN_ID),
                        managed_biguint!(500_000_000_000),
                    );
                    sc.set_accumulated_burned_tokens(
                        managed_token_id!(ETH_TOKEN_ID),
                        managed_biguint!(500_000_000_000),
                    );

                    sc.set_max_tx_batch_block_duration(100);
                },
            )
            .assert_ok();

        b_mock.set_esdt_local_roles(
            esdt_safe_wrapper.address_ref(),
            WEGLD_TOKEN_ID,
            &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
        );
        b_mock.set_esdt_local_roles(
            esdt_safe_wrapper.address_ref(),
            ETH_TOKEN_ID,
            &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
        );

        // Multi-Transfer setup

        b_mock
            .execute_tx(
                multisig_wrapper.address_ref(),
                &multi_transfer_wrapper,
                &rust_zero,
                |sc| {
                    sc.init();

                    sc.set_max_tx_batch_block_duration(3_600);
                    sc.set_esdt_safe_contract_address(OptionalValue::Some(managed_address!(
                        esdt_safe_wrapper.address_ref()
                    )));
                },
            )
            .assert_ok();

        b_mock.set_esdt_local_roles(
            multi_transfer_wrapper.address_ref(),
            WEGLD_TOKEN_ID,
            &[EsdtLocalRole::Mint],
        );
        b_mock.set_esdt_local_roles(
            multi_transfer_wrapper.address_ref(),
            ETH_TOKEN_ID,
            &[EsdtLocalRole::Mint],
        );

        // Multisig setup

        let relayer_1 = b_mock.create_user_account(&rust_zero);
        let relayer_2 = b_mock.create_user_account(&rust_zero);
        let user = b_mock.create_user_account(&rust_zero);

        b_mock.set_egld_balance(&relayer_1, &rust_biguint!(STAKE_AMOUNT));
        b_mock.set_egld_balance(&relayer_2, &rust_biguint!(STAKE_AMOUNT));
        b_mock.set_esdt_balance(&user, WEGLD_TOKEN_ID, &rust_biguint!(100_000_000_000));
        b_mock.set_esdt_balance(&user, ETH_TOKEN_ID, &rust_biguint!(200_000_000_000));

        b_mock
            .execute_tx(&owner_addr, &multisig_wrapper, &rust_zero, |sc| {
                let mut board = MultiValueEncoded::new();
                board.push(managed_address!(&relayer_1));
                board.push(managed_address!(&relayer_2));

                sc.init(
                    managed_address!(esdt_safe_wrapper.address_ref()),
                    managed_address!(multi_transfer_wrapper.address_ref()),
                    managed_biguint!(STAKE_AMOUNT),
                    managed_biguint!(500),
                    2,
                    board,
                );

                sc.unpause_endpoint();
            })
            .assert_ok();

        b_mock
            .execute_tx(
                &relayer_1,
                &multisig_wrapper,
                &rust_biguint!(STAKE_AMOUNT),
                |sc| {
                    sc.stake();
                },
            )
            .assert_ok();

        b_mock
            .execute_tx(
                &relayer_2,
                &multisig_wrapper,
                &rust_biguint!(STAKE_AMOUNT),
                |sc| {
                    sc.stake();
                },
            )
            .assert_ok();

        // Do I need EGLD-ESDT swap?

        Self {
            b_mock,
            owner_addr,
            price_agg_wrapper,
            esdt_safe_wrapper,
            multi_transfer_wrapper,
            multisig_wrapper,
        }
    }
}
