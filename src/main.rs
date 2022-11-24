// use rust_peer;
use created_swarm::{make_swarms_with_cfg, SwarmConfig};
use multiaddr::Multiaddr;

const KRAS_4: &str = "/dns4/stage.fluence.dev/tcp/19003/wss";

fn main() {
    let maddr = KRAS_4.parse::<Multiaddr>();
    if maddr.is_ok() {
        let maddr = maddr.unwrap();
        let bootstraps: Vec<Multiaddr> = vec![maddr.clone()];
        let cfg = |_cfg: SwarmConfig| SwarmConfig::new(bootstraps.clone(), maddr.clone());
        let swarm = make_swarms_with_cfg(1, cfg);
        println!("swarm {:?}", swarm);
    } else {
        println!("mulltiaddr error: {:?}", maddr);
    }
}
