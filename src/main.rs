use crate::jq::Jq;
use crate::config_file::ConfigFile;
use clap::{AppSettings, Clap};
use rocket::http::Status;
use rocket::response::{content, status};

#[macro_use] extern crate rocket;

mod payload;
mod config_file;
mod prom_metric;
mod prom_label;
mod jq;

#[derive(Clap)]
#[clap(version = "1.0", author = "Epsagon")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    json_endpoint: String,

    // Path to overrides yaml file. Optional
    #[clap(short='c', long="config", value_name="File")]
    overrides :Option<String>,

    #[clap(short='e', long="entrypoint", value_name="Entry Point in jq notation (e.g. \".components\")")]
    entry_point: Option<String>
}

async fn fetch_json(json_endpoint: String, json_entry_point: Option<String>) -> Result<String, Box<dyn std::error::Error>> {
    let res = reqwest::get(json_endpoint).await?;
    let body = res.text().await?;
    if let Some(json_entry_point) = json_entry_point {
        let jq = Jq::new()?;
        let json = jq.resolve(&body, &json_entry_point).expect("Failed to convert JSON");
        Ok(json)
    }
    else {
        Ok(body)
    }

}

fn process_json(body: String) -> Option<String> {
    let json_payload = payload::Payload::new(body);
    if let Ok(converted_metrics) = json_payload.json_to_metrics() {
        Some(converted_metrics
            .into_iter()
            .map(|metric| metric.to_string())
            .collect::<Vec<_>>()
            .join("\n"))
    }
    else {
        None
    }
}

#[get("/metrics")]
async fn metrics() -> status::Custom<content::Plain<String>> {
    let opts: Opts = Opts::parse();
    match fetch_json(opts.json_endpoint.to_string(), opts.entry_point).await {
        Ok(body) => {
            let error_message = format!("Endpoint {} provided invalid JSON\n", opts.json_endpoint);
            process_json(body).map_or(status::Custom(Status::InternalServerError, content::Plain(error_message)),
                |metrics| status::Custom(Status::Ok, content::Plain(metrics)))
        },
        Err(err) => {
            status::Custom(Status::InternalServerError, content::Plain(err.to_string()))
        }
    }
}

fn validate_config_file(opts: &Opts) {
    if let Some(config_path) = &opts.overrides {
        if let Err(err) = ConfigFile::validate_config_file(&config_path) {
            eprintln!("ERR while loading config file: {:?}", err);
            std::process::exit(1)
        }
    }
}

fn check_jq_exists() {
    if let Err(err) = jq::Jq::new() {
        eprintln!("{}", err);
        std::process::exit(1)
    }
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let opts: Opts = Opts::parse();
    println!("reading {}", opts.json_endpoint);
    check_jq_exists();

    rocket::build()
    .mount("/", routes![metrics])
    .launch()
    .await
}