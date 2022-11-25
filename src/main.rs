use std::convert::identity;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_std::future::timeout;
use async_std::task::block_on;
use created_swarm::{create_swarm, CreatedSwarm, make_swarms_with, make_swarms_with_cfg, SwarmConfig};
use fluence_libp2p::random_multiaddr::{create_memory_maddr, create_tcp_maddr};
use fluence_libp2p::Transport;
use futures::channel::oneshot::channel;
use futures::future::BoxFuture;
use futures::FutureExt;
use maplit::hashmap;
use multiaddr::Multiaddr;
use particle_execution::FunctionOutcome;
use particle_protocol::Particle;
use uuid_utils::uuid;

const STAGE_3: &str = "/dns4/stage.fluence.dev/tcp/19003/wss";
const STAGE_3_PEER_ID: &str = "12D3KooWMMGdfVEJ1rWe1nH1nehYDzNEHhg5ogdfiGk88AupCMnf";

fn main() {
    enable_logs();

    let maddr = STAGE_3.parse::<Multiaddr>();
    if maddr.is_ok() {
        let maddr = maddr.unwrap();
        let bootstraps: Vec<Multiaddr> = vec![maddr.clone()];

        let mut swarm = make_swarms_with(
            1,
            |_bs, _maddr| {
                let mut cfg = SwarmConfig::new(bootstraps.clone(), create_tcp_maddr());
                cfg.transport_timeout = Duration::from_secs(10);
                cfg.keep_alive_timeout = Duration::from_secs(10);
                create_swarm(cfg)
            },
            create_tcp_maddr,
            identity,
            true,
        ).remove(0);

        std::thread::sleep(Duration::from_secs(10));

        send_particle_to_kras_and_back(&mut swarm);
    } else {
        println!("mulltiaddr error: {:?}", maddr);
    }
}

fn send_particle_to_kras_and_back(swarm: &mut CreatedSwarm) {
    // create a kind of a promise, but for multiple values.
    // result is written via 'outlet' by one party, and then could be read via 'inlet' by another
    let (outlet, inlet) = channel();
    let mut outlet = Some(outlet);

    // define a service ('op' 'return') that will complete the above promise (via writing to outlet)
    // first, define a closure that implements the service
    let closure: Box<
        dyn FnMut(_, _) -> BoxFuture<'static, FunctionOutcome> + 'static + Send + Sync,
    > = Box::new(move |args, params| {
        let mut outlet = outlet.take();
        async move {
            let outlet = outlet.take();
            println!("got return call!!! {:?} {:?}", args, params);
            outlet.map(|out| out.send((args, params)));
            FunctionOutcome::Empty
        }
        .boxed()
    });

    let add_service_f = swarm
        .aquamarine_api
        .clone()
        .add_service("op".into(), hashmap! { "return".to_string() => closure });

    let script = format!(
        r#"
        (seq
            (call "{}" ("op" "identity") ["hello"] result)
            (call %init_peer_id% ("op" "return") [result])
        )
    "#,
        STAGE_3_PEER_ID
    );

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before Unix epoch").as_millis() as u64;
    let particle = Particle {
        id: uuid(),
        init_peer_id: swarm.peer_id,
        timestamp: now,
        ttl: 25000,
        script,
        signature: vec![],
        data: vec![],
    };

    println!("particle id is {}", particle.id);

    // send particle to execution
    let exec_f = swarm.aquamarine_api.clone().execute(particle, None);

    let result = block_on(timeout(Duration::from_secs(30), async move {
        println!("before add_service_f");
        println!("add_service_f done: {:?}", add_service_f.await);
        println!("before exec_f");
        println!("exec_f done: {:?}", exec_f.await);
        println!("before inlet");
        inlet.await
    }));

    println!("result: {:?}", result);
}

fn enable_logs() {
    use log::LevelFilter::*;

    std::env::set_var("WASM_LOG", "info");

    env_logger::builder()
        .format_timestamp_millis()
        .filter_level(log::LevelFilter::Debug)
        .filter(Some("script_storage"), Trace)
        .filter(Some("aquamarine"), Trace)
        .filter(Some("network"), Trace)
        .filter(Some("network_api"), Trace)
        .filter(Some("aquamarine::actor"), Debug)
        .filter(Some("particle_node::bootstrapper"), Info)
        .filter(Some("yamux::connection::stream"), Info)
        .filter(Some("tokio_threadpool"), Info)
        .filter(Some("tokio_reactor"), Info)
        .filter(Some("mio"), Info)
        .filter(Some("tokio_io"), Info)
        .filter(Some("soketto"), Info)
        .filter(Some("yamux"), Info)
        .filter(Some("multistream_select"), Info)
        .filter(Some("libp2p_swarm"), Info)
        .filter(Some("libp2p_secio"), Info)
        .filter(Some("libp2p_websocket::framed"), Info)
        .filter(Some("libp2p_ping"), Info)
        .filter(Some("libp2p_core::upgrade::apply"), Info)
        .filter(Some("libp2p_kad::kbucket"), Info)
        .filter(Some("libp2p_kad"), Info)
        .filter(Some("libp2p_kad::query"), Info)
        .filter(Some("libp2p_kad::iterlog"), Info)
        .filter(Some("libp2p_plaintext"), Info)
        .filter(Some("libp2p_identify::protocol"), Info)
        .filter(Some("cranelift_codegen"), Info)
        .filter(Some("wasmer_wasi"), Info)
        .filter(Some("wasmer_interface_types_fl"), Info)
        .filter(Some("async_std"), Info)
        .filter(Some("async_io"), Info)
        .filter(Some("polling"), Info)
        .filter(Some("cranelift_codegen"), Info)
        .filter(Some("walrus"), Info)
        .try_init()
        .unwrap();
}
