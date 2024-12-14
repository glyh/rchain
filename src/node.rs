use std::collections::HashSet;

use actix::prelude::*;

use crate::blockchain::{Block, Chain};

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
struct KeepMining;

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

impl Handler<KeepMining> for Node {
    type Result = ();
    fn handle(&mut self, msg: KeepMining, ctx: &mut Self::Context) -> Self::Result {}
}
