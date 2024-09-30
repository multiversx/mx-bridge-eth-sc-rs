use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    ScenarioWorld::vm_go()
}

#[test]
fn add_wrapped_token_go() {
    world().run("scenarios/add_wrapped_token.scen.json");
}

#[test]
#[ignore] //Ignore for now
fn blacklist_token_go() {
    world().run("scenarios/blacklist_token.scen.json");
}

#[test]
#[ignore] //Ignore for now
fn remove_wrapped_token_go() {
    world().run("scenarios/remove_wrapped_token.scen.json");
}

#[test]
fn setup_go() {
    world().run("scenarios/setup.scen.json");
}

#[test]
#[ignore] //Ignore for now
fn unwrap_token_go() {
    world().run("scenarios/unwrap_token.scen.json");
}

#[test]
fn whitelist_token_go() {
    world().run("scenarios/whitelist_token.scen.json");
}

#[test]
fn wrap_token_go() {
    world().run("scenarios/wrap_token.scen.json");
}
