//! Example of how to trace ALL transaction pool events including invalid transactions
//!
//! Run with
//!
//! ```sh
//! cargo run --release -p example-txpool-tracing --bin full-events -- node --http --ws
//! ```

use clap::Parser;
use futures_util::StreamExt;
use reth_ethereum::{
    cli::{chainspec::EthereumChainSpecParser, interface::Cli},
    node::{builder::NodeHandle, EthereumNode},
};
use reth_transaction_pool::{FullTransactionEvent, TransactionPool};

#[derive(Debug, Clone, Default, clap::Args)]
struct Args;

fn main() {
    Cli::<EthereumChainSpecParser, Args>::parse()
        .run(|builder, _args| async move {
            let NodeHandle { node, node_exit_future } =
                builder.node(EthereumNode::default()).launch().await?;

            // Subscribe to ALL pool events (including invalid transactions)
            let mut pool_events = node.pool.all_transactions_event_listener();

            println!("🎯 Listening for ALL transaction pool events...");

            node.task_executor.spawn(Box::pin(async move {
                while let Some(event) = pool_events.next().await {
                    match event {
                        FullTransactionEvent::Pending(hash) => {
                            println!("✅ PENDING: {hash}");
                        }
                        FullTransactionEvent::Queued(hash) => {
                            println!("⏳ QUEUED: {hash}");
                        }
                        FullTransactionEvent::Invalid(hash) => {
                            println!("❌ INVALID: {hash} (failed validation)");
                        }
                        FullTransactionEvent::Discarded(hash) => {
                            println!("🗑️  DISCARDED: {hash} (limits exceeded)");
                        }
                        FullTransactionEvent::Replaced { transaction, replaced_by } => {
                            println!("🔄 REPLACED: {} → {replaced_by}", transaction.hash());
                        }
                        FullTransactionEvent::Mined { tx_hash, block_hash } => {
                            println!("⛏️  MINED: {tx_hash} in block {block_hash}");
                        }
                        FullTransactionEvent::Propagated(_peers) => {
                            println!("📡 PROPAGATED to network");
                        }
                    }
                }
            }));

            node_exit_future.await
        })
        .unwrap();
}
