//! ClawChain CLI configuration.

use sc_cli::RunCmd;

#[derive(Debug, clap::Parser)]
#[command(
    name = "clawchain-node",
    about = "ClawChain: Agent-native Layer 1 blockchain node",
    version
)]
pub struct Cli {
    /// The subcommand to run.
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,

    /// Run the node.
    #[clap(flatten)]
    pub run: RunCmd,
}

/// Available subcommands.
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// Key management CLI utilities.
    #[command(subcommand)]
    Key(sc_cli::KeySubcommand),

    /// Build a chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Remove the whole chain.
    PurgeChain(sc_cli::PurgeChainCmd),

    /// Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),

    /// Db meta columns information.
    ChainInfo(sc_cli::ChainInfoCmd),
}
