#[test]
fn unwrap_token_go() {
    multiversx_sc_scenario::run_go("mandos/unwrap_token.scen.json");
}

#[test]
fn wrap_token_go() {
    multiversx_sc_scenario::run_go("mandos/wrap_token.scen.json");
}

#[test]
fn whitelist_token_go() {
    multiversx_sc_scenario::run_go("mandos/whitelist_token.scen.json");
}

#[test]
fn blacklist_token_go() {
    multiversx_sc_scenario::run_go("mandos/blacklist_token.scen.json");
}

#[test]
fn add_wrapped_token_go() {
    multiversx_sc_scenario::run_go("mandos/add_wrapped_token.scen.json");
}

#[test]
fn remove_wrapped_token_go() {
    multiversx_sc_scenario::run_go("mandos/remove_wrapped_token.scen.json");
}
