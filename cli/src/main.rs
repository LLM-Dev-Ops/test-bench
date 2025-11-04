use clap::{Parser, Subcommand};
use std::process;

mod commands;
mod output;

use commands::{bench, config, eval, test};

/// LLM Test Bench - Production-grade CLI for testing and benchmarking LLM applications
#[derive(Parser)]
#[command(name = "llm-test-bench")]
#[command(author = "LLM Test Bench Contributors")]
#[command(version)]
#[command(about = "A production-grade CLI for testing and benchmarking LLM applications", long_about = None)]
#[command(propagate_version = true)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a single test against an LLM provider
    #[command(visible_alias = "t")]
    Test(test::TestArgs),

    /// Run benchmark tests across multiple providers
    #[command(visible_alias = "b")]
    Bench(bench::BenchArgs),

    /// Evaluate test results with metrics
    #[command(visible_alias = "e")]
    Eval(eval::EvalArgs),

    /// Configuration management commands
    #[command(subcommand)]
    Config(config::ConfigCommands),

    /// Generate shell completions
    Completions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    // Handle color output
    if cli.no_color {
        colored::control::set_override(false);
    }

    // Execute command
    let result = match cli.command {
        Commands::Test(args) => test::execute(args, cli.verbose).await,
        Commands::Bench(args) => bench::execute(args, cli.verbose).await,
        Commands::Eval(args) => eval::execute(args, cli.verbose).await,
        Commands::Config(cmd) => config::execute(cmd, cli.verbose).await,
        Commands::Completions { shell } => {
            generate_completions(shell);
            Ok(())
        }
    };

    // Handle errors
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        if cli.verbose {
            eprintln!("\nCaused by:");
            for cause in e.chain().skip(1) {
                eprintln!("  {}", cause);
            }
        }
        process::exit(1);
    }
}

fn generate_completions(shell: clap_complete::Shell) {
    use clap::CommandFactory;
    use clap_complete::generate;
    use std::io;

    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    generate(shell, &mut cmd, bin_name, &mut io::stdout());
}

