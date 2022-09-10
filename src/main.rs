use anyhow::Context;
use clap::Parser;
use log::LevelFilter;
use std::fs;

mod api;
mod models;
mod player;
mod storage;
mod ui;

/// TODO: add about.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Opt {
    /// Logging level
    #[clap(long, default_value = "error")]
    level: LevelFilter,

    /// Log file path (for debugging)
    #[clap(long, default_value = ".rtap.log")]
    log_file: String,

    /// Radio browser address
    #[clap(long, default_value = "https://de1.api.radio-browser.info")]
    radio_browser_url: String,

    /// SQLite database path
    #[clap(long, default_value = "tradio.db")]
    db_path: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    let log_file = fs::File::create(&opt.log_file).context("can't open log file")?;

    simplelog::WriteLogger::init(opt.level, simplelog::Config::default(), log_file)
        .context("init logger")?;

    let player = player::Rodio::default()?;
    let storage = storage::Sqlite::new(&opt.db_path).await?;
    let client = api::RadioBrowser::new(&opt.radio_browser_url);

    ui::Ui::new(player, storage, client).start().await
}
