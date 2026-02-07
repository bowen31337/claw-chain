//! ClawChain Node - Agent-native Layer 1 blockchain
//!
//! This is the main entry point for the ClawChain node binary.

#![warn(missing_docs)]

mod chain_spec;
mod cli;
mod command;
mod rpc;
mod service;

fn main() -> sc_cli::Result<()> {
    command::run()
}
