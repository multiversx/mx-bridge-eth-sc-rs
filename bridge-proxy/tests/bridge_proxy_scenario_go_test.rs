use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    ScenarioWorld::vm_go()
}

#[test]
fn bridge_proxy_go() {
    world().run("scenarios/bridge-proxy.scen.json");
}
