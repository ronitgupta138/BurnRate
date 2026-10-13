//! burnrate CLI: Terminal HUD, Hard Spend Caps & Cache Guard.

use burnrate::budget::BudgetGovernor;
use burnrate::cache_linter::PromptCacheLinter;
use burnrate::hud::TerminalHud;
use burnrate::pricing::PricingRegistry;
use burnrate::proxy::{create_proxy_router, ProxyState};
use burnrate::secret_guard::SecretGuard;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::net::SocketAddr;
use std::process::Command;
use std::sync::Arc;

#[derive(Parser)]
#[command(
    name = "burnrate",
    author = "Ronit Gupta <ronitgupta138@gmail.com>",
    version = "1.2.0",
    about = "Terminal HUD, Hard Spend Caps & Cache Guard for Coding Agents"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the reverse proxy daemon with hard spend cap
    Serve {
        /// Maximum session spend in USD (default: 2.00)
        #[arg(long, default_value_t = 2.00)]
        max_spend: f64,

        /// Port to bind proxy on (default: 7070)
        #[arg(long, default_value_t = 7070)]
        port: u16,

        /// Host address (default: 127.0.0.1)
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },

    /// Run an agent command wrapped with automatic budget circuit breaker
    Run {
        /// Maximum budget spend in USD (e.g. 1.50)
        #[arg(long, default_value_t = 2.00)]
        max_spend: f64,

        /// Port to run proxy on (default: 7070)
        #[arg(long, default_value_t = 7070)]
        port: u16,

        /// Command and arguments to execute
        #[arg(required = true, num_args = 1..)]
        cmd: Vec<String>,
    },

    /// Inspect pricing tables for model family
    Pricing {
        /// Model name (e.g. claude-sonnet, gpt-4o, deepseek-v3)
        #[arg(long, default_value = "claude-sonnet")]
        model: String,
    },

    /// Render sample terminal HUD status bar
    Hud,

    /// Run native Model Context Protocol (MCP) JSON-RPC 2.0 stdio server
    Mcp {
        /// Maximum budget spend in USD (default: 2.00)
        #[arg(long, default_value_t = 2.00)]
        max_spend: f64,
    },

    /// View historical session ledger and spending analytics
    History {
        /// Maximum number of records to show (default: 10)
        #[arg(long, default_value_t = 10)]
        limit: usize,

        /// Show cumulative spending summary across all historical sessions
        #[arg(long)]
        summary: bool,
    },

    /// Initialize configuration and detect installed coding agents
    Init,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { max_spend, port, host } => {
            let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
            let governor = Arc::new(BudgetGovernor::new(max_spend));
            let linter = Arc::new(PromptCacheLinter::new());
            let secret_guard = Arc::new(SecretGuard::new());
            let ledger = Arc::new(burnrate::ledger::Ledger::new(burnrate::ledger::Ledger::default_path()));
            let state = Arc::new(ProxyState::with_ledger(governor, linter, secret_guard, ledger));

            let app = create_proxy_router(state);

            println!();
            println!("{}", "┌────────────────────────────────────────────────────────────────────────┐".cyan());
            println!(
                "{}  {}  {}",
                "│".cyan(),
                "⚡ burnrate Cost Shield & Prompt-Cache Proxy".bold(),
                "                    │".cyan()
            );
            println!("{}", "├────────────────────────────────────────────────────────────────────────┤".cyan());
            println!(
                "{}  {: <14} :: {: <50} {}",
                "│".cyan(),
                "Proxy Port".dimmed(),
                format!("http://{}:{}/v1", host, port).green().bold(),
                "│".cyan()
            );
            println!(
                "{}  {: <14} :: {: <50} {}",
                "│".cyan(),
                "Hard Cap".dimmed(),
                format!("${:.2} USD (Auto-Kill Circuit Breaker)", max_spend).yellow().bold(),
                "│".cyan()
            );
            println!("{}", "└────────────────────────────────────────────────────────────────────────┘".cyan());
            println!();

            let listener = tokio::net::TcpListener::bind(addr).await?;
            axum::serve(listener, app).await?;
        }

        Commands::Run { max_spend, port, cmd } => {
            println!(
                "{} Starting burnrate budget guard (Hard Spend Limit: ${:.2})",
                "⚡".yellow().bold(),
                max_spend
            );

            let proxy_url = format!("http://127.0.0.1:{}", port);
            let program = &cmd[0];
            let args = &cmd[1..];

            let mut child = Command::new(program)
                .args(args)
                .env("ANTHROPIC_BASE_URL", &proxy_url)
                .env("OPENAI_BASE_URL", &proxy_url)
                .spawn()?;

            let status = child.wait()?;
            println!(
                "{} Process `{}` exited with status: {}",
                "✔".green().bold(),
                program.cyan(),
                status
            );
        }

        Commands::Pricing { model } => {
            let pricing = PricingRegistry::lookup(&model);
            println!();
            println!("Pricing Rates for {}:", pricing.model_id.cyan().bold());
            println!("  Input Tokens:       ${:.2} / MTok", pricing.input_per_million);
            println!("  Output Tokens:      ${:.2} / MTok", pricing.output_per_million);
            println!("  Cache Read (90% off): ${:.2} / MTok", pricing.cache_read_per_million);
            println!("  Cache Write:        ${:.2} / MTok", pricing.cache_write_per_million);
            println!();
        }

        Commands::Hud => {
            let gov = BudgetGovernor::new(2.00);
            gov.record_turn(0.42, 18400, 1200, 16900);
            let status = gov.status();
            TerminalHud::display(&status);
        }

        Commands::Mcp { max_spend } => {
            let mcp = burnrate::mcp::McpServer::new(max_spend);
            let stdin = std::io::stdin();
            let stdout = std::io::stdout();
            mcp.run_loop(stdin.lock(), stdout.lock())?;
        }

        Commands::History { limit, summary } => {
            let ledger = burnrate::ledger::Ledger::new(burnrate::ledger::Ledger::default_path());
            if summary {
                let s = ledger.summary()?;
                println!();
                println!("{}", "⚡ burnrate Historical Ledger Summary".bold().cyan());
                println!("==========================================");
                println!("  Total Sessions:        {}", s.total_sessions);
                println!("  Total Spend:           ${:.2} USD", s.total_spent_usd);
                println!("  Input Tokens:          {}", s.total_input_tokens);
                println!("  Output Tokens:         {}", s.total_output_tokens);
                println!("  Cache Read Tokens:     {}", s.total_cache_read_tokens);
                println!("  Avg Cache Hit Rate:    {:.1}%", s.avg_cache_hit_percent);
                println!("  Secrets Redacted:      {}", s.total_secrets_redacted);
                println!("  Tripped Sessions:      {}", s.tripped_sessions);
                println!();
            } else {
                let records = ledger.list_records(limit)?;
                println!();
                println!("{}", "⚡ burnrate Recent Agent Sessions".bold().cyan());
                println!("{}", burnrate::ledger::Ledger::render_table(&records));
                println!();
            }
        }

        Commands::Init => {
            let cfg = burnrate::config::BurnrateConfig::load();
            cfg.save()?;
            println!();
            println!("{}", "⚡ burnrate Initializer & Agent Detector".bold().cyan());
            println!("==========================================");
            println!("Configuration saved to: {}", burnrate::config::BurnrateConfig::default_path().display().to_string().green());
            println!();
            println!("{}", "Detected AI Coding Agents in PATH:".bold());
            let detected = burnrate::config::BurnrateConfig::detect_installed_agents();
            if detected.is_empty() {
                println!("  (No standard agent CLIs detected in PATH yet)");
            } else {
                for agent in &detected {
                    println!("  ✓ {: <10} :: {: <30} ({})", agent.binary.green().bold(), agent.name, agent.path.dimmed());
                }
            }
            println!();
            println!("{}", "Recommended Shell Aliases (add to ~/.bashrc or ~/.zshrc):".bold());
            println!("  alias claude=\"burnrate run --max-spend {:.2} -- claude\"", cfg.default_max_spend);
            println!("  alias cursor=\"burnrate run --max-spend {:.2} -- cursor\"", cfg.default_max_spend);
            println!("  alias opencode=\"burnrate run --max-spend {:.2} -- opencode\"", cfg.default_max_spend);
            println!("  alias aider=\"burnrate run --max-spend {:.2} -- aider\"", cfg.default_max_spend);
            println!();
        }
    }

    Ok(())
}
