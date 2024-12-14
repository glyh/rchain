use std::{cmp, collections::HashSet, time::Duration};

use actix::prelude::*;
use rand::thread_rng;

use crate::{
    blockchain::{Block, Chain},
    rand_str::rand_string,
};

/// Message Definitons

type NodeAddr = Addr<Node>;

#[derive(Message)]
#[rtype(result = "()")]
struct Ping(NodeAddr);

#[derive(Message)]
#[rtype(result = "()")]
struct Register(NodeAddr);

#[derive(Message)]
#[rtype(result = "()")]
struct RequestLatestBlock(NodeAddr);

#[derive(Message)]
#[rtype(result = "()")]
struct ResponseNewBlock {
    block: Block,
    source: NodeAddr,
}

#[derive(Message)]
#[rtype(result = "()")]
struct RequestWholeChain(NodeAddr);

#[derive(Message)]
#[rtype(result = "()")]
struct ResponseWholeChain(Chain);

#[derive(Message)]
#[rtype(result = "()")]
struct Mine;

#[derive(Message)]
#[rtype(result = "()")]
struct Pulse(Duration);

/// Actor Definitions

struct Node {
    chain: Chain,
    known_peers: HashSet<NodeAddr>,
}

impl Actor for Node {
    type Context = Context<Self>;
}

impl Handler<Ping> for Node {
    type Result = ();
    fn handle(&mut self, msg: Ping, ctx: &mut Self::Context) -> Self::Result {
        let _ = msg.0.send(Register(ctx.address()));
        self.known_peers.insert(msg.0);
    }
}

impl Handler<Register> for Node {
    type Result = ();
    fn handle(&mut self, msg: Register, _ctx: &mut Self::Context) -> Self::Result {
        self.known_peers.insert(msg.0);
    }
}

impl Handler<RequestLatestBlock> for Node {
    type Result = ();
    fn handle(&mut self, msg: RequestLatestBlock, ctx: &mut Context<Self>) -> Self::Result {
        let _ = msg.0.send(ResponseNewBlock {
            block: self.chain.get_latest().clone(),
            source: ctx.address(),
        });
    }
}

impl Handler<ResponseNewBlock> for Node {
    type Result = ();
    fn handle(&mut self, msg: ResponseNewBlock, ctx: &mut Self::Context) -> Self::Result {
        let new_block = msg.block;
        if self.chain.try_append(new_block) {
            return;
        } else {
            let _ = msg.source.send(RequestWholeChain(ctx.address()));
        }
    }
}

impl Handler<RequestWholeChain> for Node {
    type Result = ();
    fn handle(&mut self, msg: RequestWholeChain, _ctx: &mut Context<Self>) -> Self::Result {
        let _ = msg.0.send(ResponseWholeChain(self.chain.clone()));
    }
}

impl Handler<ResponseWholeChain> for Node {
    type Result = ();
    fn handle(&mut self, msg: ResponseWholeChain, _ctx: &mut Context<Self>) -> Self::Result {
        self.chain.try_replace_with(msg.0);
    }
}

impl Handler<Mine> for Node {
    type Result = ();
    fn handle(&mut self, _msg: Mine, ctx: &mut Self::Context) -> Self::Result {
        let generated = self.chain.generate_block(rand_string(6));
        // notify a bunch of neighbors
        for peer in self.select_peers() {
            let _ = peer.send(ResponseNewBlock {
                block: generated.clone(),
                source: ctx.address(),
            });
        }
    }
}

const PULSE_DURATION: Duration = Duration::new(20, 0);

// NOTE: in additional to broadcasting,
// learn from peers regularly as well

impl Handler<Pulse> for Node {
    type Result = ();
    fn handle(&mut self, msg: Pulse, ctx: &mut Self::Context) -> Self::Result {
        for peer in self.select_peers() {
            let _ = peer.send(RequestWholeChain(ctx.address()));
        }
        ctx.run_later(msg.0, move |_, ctx| {
            let _ = ctx.address().send(msg);
        });
    }
}

const WHISPER_LIMIT: usize = 4;

impl Node {
    fn select_peers(&self) -> HashSet<NodeAddr> {
        let num_to_sample = cmp::min(self.known_peers.len(), WHISPER_LIMIT);
        let mut rng = thread_rng();
        let peer_cnt = self.known_peers.len();
        let vec: Vec<&NodeAddr> = self.known_peers.iter().collect();
        let mut output = HashSet::new();
        for idx in rand::seq::index::sample(&mut rng, peer_cnt, num_to_sample) {
            output.insert(vec[idx].clone());
        }
        output
    }
}
