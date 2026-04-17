// unbill-cli: command-line frontend.
// See DESIGN.md for the subcommand tree. Implementation begins at M2.

use clap::Parser;

#[derive(Parser)]
#[command(name = "unbill", about = "Peer-to-peer bill splitting.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Initialize unbill data directory on this device.
    Init,
    /// Manage ledgers.
    Ledger {
        #[command(subcommand)]
        sub: LedgerCmd,
    },
    /// Manage bills.
    Bill {
        #[command(subcommand)]
        sub: BillCmd,
    },
    /// Manage members.
    Member {
        #[command(subcommand)]
        sub: MemberCmd,
    },
    /// Sync with peers.
    Sync {
        #[command(subcommand)]
        sub: SyncCmd,
    },
    /// Show settlement summary.
    Settlement { ledger_id: String },
}

#[derive(clap::Subcommand)]
enum LedgerCmd {
    Create { name: String, currency: String },
    List,
    Show { ledger_id: String },
    Export { ledger_id: String, output: String },
    Import { file: String },
    Delete { ledger_id: String },
}

#[derive(clap::Subcommand)]
enum BillCmd {
    Add,
    List { ledger_id: String },
    Amend { ledger_id: String, bill_id: String },
    Delete { ledger_id: String, bill_id: String },
    Restore { ledger_id: String, bill_id: String },
}

#[derive(clap::Subcommand)]
enum MemberCmd {
    List { ledger_id: String },
    Invite { ledger_id: String },
    Join { url: String },
}

#[derive(clap::Subcommand)]
enum SyncCmd {
    Daemon,
    Once { ledger_id: String },
    Status,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _cli = Cli::parse();
    println!("unbill CLI — implementation begins at M2");
    Ok(())
}
