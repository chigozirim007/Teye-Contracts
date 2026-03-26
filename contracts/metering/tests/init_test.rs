#![allow(clippy::unwrap_used, clippy::expect_used)]

use metering::{
    GasCosts, MeteringContract, MeteringContractClient, MeteringError, OperationType, TenantLevel,
};
use soroban_sdk::{testutils::Address as _, Address, Env};

fn uninitialized_setup() -> (Env, MeteringContractClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(MeteringContract, ());
    let client = MeteringContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let outsider = Address::generate(&env);

    (env, client, admin, outsider)
}

#[test]
fn initialize_sets_initial_admin_and_default_cost_constraints() {
    let (_env, client, admin, _outsider) = uninitialized_setup();

    client.initialize(&admin);

    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.get_gas_costs(), GasCosts::default_costs());
}

#[test]
fn stateful_entrypoints_reject_calls_before_initialization() {
    let (env, client, admin, outsider) = uninitialized_setup();
    let tenant = Address::generate(&env);
    let costs = GasCosts {
        read_cost: 2,
        write_cost: 6,
        compute_cost: 11,
        storage_cost: 4,
    };

    assert_eq!(client.try_get_admin(), Err(Ok(MeteringError::NotInitialized)));
    assert_eq!(
        client.try_set_gas_costs(&admin, &costs),
        Err(Ok(MeteringError::NotInitialized))
    );
    assert_eq!(
        client.try_register_tenant(&admin, &tenant, &TenantLevel::Organization, &tenant),
        Err(Ok(MeteringError::NotInitialized))
    );
    assert_eq!(
        client.try_record_gas(&outsider, &tenant, &OperationType::Read),
        Err(Ok(MeteringError::NotInitialized))
    );
}

#[test]
fn second_initialize_cannot_overwrite_original_admin_or_defaults() {
    let (_env, client, admin, outsider) = uninitialized_setup();

    client.initialize(&admin);

    let reinitialize = client.try_initialize(&outsider);
    assert_eq!(reinitialize, Err(Ok(MeteringError::AlreadyInitialized)));

    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.get_gas_costs(), GasCosts::default_costs());
}

#[test]
fn post_initialize_constraints_still_enforce_admin_boundaries() {
    let (_env, client, admin, outsider) = uninitialized_setup();
    let custom_costs = GasCosts {
        read_cost: 3,
        write_cost: 7,
        compute_cost: 12,
        storage_cost: 5,
    };

    client.initialize(&admin);

    assert_eq!(
        client.try_set_gas_costs(&outsider, &custom_costs),
        Err(Ok(MeteringError::Unauthorized))
    );
    assert_eq!(client.get_gas_costs(), GasCosts::default_costs());
}
