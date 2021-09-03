use clap::{AppSettings, Clap};
use rocket::http::Status;
use rocket::response::{content, status};

#[macro_use] extern crate rocket;

mod payload;

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

#[get("/metrics")]
async fn metrics() -> status::Custom<content::Plain<String>> {
    let opts: Opts = Opts::parse();
    match fetch_json(opts.json_endpoint).await {
        Ok(body) => {
            println!("{}", body);

            let metrics = payload::json_to_metrics(body)
                        .unwrap()
                        .into_iter()
                        .map(|metric| metric.to_string())
                        .collect::<Vec<_>>()
                        .join("\n");

            status::Custom(Status::Ok, content::Plain(metrics))
        },
        Err(err) => {
            status::Custom(Status::InternalServerError, content::Plain(err.to_string()))
        }
    }
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let opts: Opts = Opts::parse();
    println!("reading {}", opts.json_endpoint);

    rocket::build()
    .mount("/", routes![metrics])
    .launch()
    .await
}


// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     println!("Prom JSON Exporter");



//     match fetch_json(opts.json_endpoint).await {
//         Ok(body) => {
//             println!("{}", body);
//             for metric in payload::json_to_metrics(body).unwrap() {
//                 println!("{}", metric.to_string());
//             }
//         },
//         Err(err) => {
//             println!("{:?}", err);
//         }
//     }

//     Ok(())
// }

