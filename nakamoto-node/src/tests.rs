use std::net;
use std::thread;

use nakamoto_chain::block::cache::BlockCache;
use nakamoto_chain::block::store;
use nakamoto_test::logger;

use crate::error;
use crate::handle::Handle;
use crate::node::{Event, Node, NodeConfig, NodeHandle};

fn network(
    size: usize,
    cfg: NodeConfig,
) -> Result<Vec<(NodeHandle, net::SocketAddr, thread::JoinHandle<()>)>, error::Error> {
    let checkpoints = cfg.network.checkpoints().collect::<Vec<_>>();
    let genesis = cfg.network.genesis();
    let params = cfg.network.params();

    let mut handles = Vec::new();

    for _ in 0..size {
        let mut node = Node::new(cfg.clone())?;
        let handle = node.handle();

        let t = thread::spawn({
            let params = params.clone();
            let checkpoints = checkpoints.clone();

            move || {
                let store = store::Memory::new((genesis, vec![]).into());
                let cache = BlockCache::from(store, params, &checkpoints).unwrap();

                node.run_with(cache).unwrap();
            }
        });

        let addr = handle
            .wait_for(|e| match e {
                Event::Listening(addr) => Some(addr),
                _ => None,
            })
            .unwrap();

        handles.push((handle, addr, t));
    }

    for (handle, addr, _) in &handles {
        for (_, peer, _) in &handles {
            if peer != addr {
                handle.connect(*peer).unwrap();
            }
        }
    }

    Ok(handles)
}

#[test]
fn test_full_sync() {
    logger::init(log::Level::Debug);

    network(3, NodeConfig::default()).unwrap();
}
