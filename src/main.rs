// use rust_peer;
use created_swarm::{make_swarms_with_cfg, SwarmConfig};
use multiaddr::Multiaddr;

const KRAS_4: &str = "/dns4/kras-02.fluence.dev/tcp/19001/wss/p2p/12D3KooWHLxVhUQyAuZe6AHMB29P7wkvTNMn7eDMcsqimJYLKREf";

fn main() {
    let maddr = KRAS_4.parse::<Multiaddr>();
    if maddr.is_ok() {
        let maddr = maddr.unwrap();
        let bootstraps: Vec<Multiaddr> = vec![maddr.clone()];
        let cfg = |_cfg: SwarmConfig| SwarmConfig::new(bootstraps.clone(), maddr.clone());
        let swarm = make_swarms_with_cfg(1, cfg);
        println!("{:?}", swarm);
    }
}
