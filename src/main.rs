use clap::{AppSettings, Clap};


/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(version = "1.0", author = "Epsagon")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    json_endpoint: String
}

fn main() {
    println!("Prom JSON Exporter");
    let opts: Opts = Opts::parse();

    println!("reading {}", opts.json_endpoint);
}

