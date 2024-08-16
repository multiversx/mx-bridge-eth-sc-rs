use multisig::Multisig;
use multiversx_sc_scenario::imports::*;

const OWNER_ADDRESS_EXPR: &str = "address:owner";
const PROPOSER_ADDRESS_EXPR: &str = "address:proposer";
const BOARD_MEMBER_ADDRESS_EXPR: &str = "address:board-member";
const MULTISIG_ADDRESS_EXPR: &str = "sc:multisig";
const MULTISIG_PATH_EXPR: &str = "file:output/multisig.wasm";
const QUORUM_SIZE: usize = 1;

fn world() -> ScenarioWorld {
    let mut blockchain: ScenarioWorld = ScenarioWorld::new();

    blockchain.register_contract(MULTISIG_PATH_EXPR, multisig::ContractBuilder);
    blockchain
}

fn setup() -> ScenarioWorld {
    let mut world = world();

    let multisig = WhiteboxContract::new(MULTISIG_ADDRESS_EXPR, multisig::contract_obj);

    let multisig_code = world.code_expression(MULTISIG_PATH_EXPR);

    let mut set_state_step = SetStateStep::new()
        .put_account(
            OWNER_ADDRESS_EXPR,
            Account::new().nonce(1).balance(100000u64),
        )
        .new_address(OWNER_ADDRESS_EXPR, 1, MULTISIG_ADDRESS_EXPR)
        .block_timestamp(100);

    let esdt_safe_address_expr = "sc:esdt_safe".to_string();
    let esdt_safe_address = AddressValue::from(esdt_safe_address_expr.as_str());

    let multi_transfer_address_expr = "sc:multi_transfer_esdt".to_string();
    let multi_transfer_address = AddressValue::from(multi_transfer_address_expr.as_str());

    let proxy_address_expr = "sc:proxy";
    let proxy_address = AddressValue::from(proxy_address_expr);

    set_state_step = set_state_step
        .put_account(esdt_safe_address_expr.as_str(), Account::new().nonce(1))
        .put_account(
            multi_transfer_address_expr.as_str(),
            Account::new().nonce(1),
        )
        .put_account(proxy_address_expr, Account::new().nonce(1))
        .put_account(BOARD_MEMBER_ADDRESS_EXPR, Account::new().nonce(1));

    world.set_state_step(set_state_step).whitebox_deploy(
        &multisig,
        ScDeployStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .code(multisig_code),
        |sc| {
            let mut board_members = ManagedVec::new();
            board_members.push(managed_address!(&address_expr_to_address(
                BOARD_MEMBER_ADDRESS_EXPR
            )));

            let required_stake = BigUint::from(1000u32);
            let slash_amount = BigUint::from(500u32);
            let quorum = 2;
            sc.init(
                managed_address!(&esdt_safe_address.to_address()),
                managed_address!(&multi_transfer_address.to_address()),
                managed_address!(&proxy_address.to_address()),
                required_stake,
                slash_amount,
                quorum,
                board_members.into(),
            );
        },
    );

    world
}

#[test]
fn test_init() {
    setup();
}

fn address_expr_to_address(address_expr: &str) -> Address {
    AddressValue::from(address_expr).to_address()
}

fn boxed_bytes_vec_to_managed<M: ManagedTypeApi>(
    raw_vec: Vec<BoxedBytes>,
) -> ManagedVec<M, ManagedBuffer<M>> {
    let mut managed = ManagedVec::new();
    for elem in raw_vec {
        managed.push(ManagedBuffer::new_from_bytes(elem.as_slice()));
    }

    managed
}
