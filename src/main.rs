use clap::{AppSettings, Clap};

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(version = "1.0", author = "Epsagon")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    json_endpoint: String
}

async fn fetch_json(json_endpoint: String) -> Result<String, Box<dyn std::error::Error>> {
    let res = reqwest::get(json_endpoint).await?;
    let body = res.text().await?;
    Ok(body)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Prom JSON Exporter");
    let opts: Opts = Opts::parse();

    println!("reading {}", opts.json_endpoint);

    match fetch_json(opts.json_endpoint).await {
        Ok(body) => {
            println!("{}", body);
        },
        Err(err) => {
            println!("{:?}", err);
        }
    }

    Ok(())
}

