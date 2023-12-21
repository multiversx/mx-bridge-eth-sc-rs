use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    todo!()
}

#[test]
fn add_wrapped_token_rs() {
    world().run("scenarios/add_wrapped_token.scen.json");
}

#[test]
fn blacklist_token_rs() {
    world().run("scenarios/blacklist_token.scen.json");
}

#[test]
fn remove_wrapped_token_rs() {
    world().run("scenarios/remove_wrapped_token.scen.json");
}

#[test]
fn setup_rs() {
    world().run("scenarios/setup.scen.json");
}

#[test]
fn unwrap_token_rs() {
    world().run("scenarios/unwrap_token.scen.json");
}

#[test]
fn whitelist_token_rs() {
    world().run("scenarios/whitelist_token.scen.json");
}

#[test]
fn wrap_token_rs() {
    world().run("scenarios/wrap_token.scen.json");
}
