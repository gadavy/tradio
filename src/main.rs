use anyhow::Context;
use clap::Parser;
use log::LevelFilter;
use std::fs;

mod api;
mod app;
mod models;
mod player;
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    let log_file = fs::File::create(&opt.log_file).context("can't open log file")?;

    simplelog::WriteLogger::init(opt.level, simplelog::Config::default(), log_file)
        .context("init logger")?;

    let client = api::Client::new(&opt.radio_browser_url);

    let player = player::RodioPlayer::default()?;
    let application = app::App::new(player, client)?;

    let mut ui = ui::Ui::new(application);

    ui.start().await
}
