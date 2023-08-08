use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use tokio::time::{sleep, Duration};
use chrono::Local;
use chrono::prelude::*;
use chrono_tz::Asia::Shanghai;

#[derive(Debug, Deserialize)]
struct Config {
    Api: String,
    SaveDir: String,
    RunningState: bool,
    Data: Vec<DataItem>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DataItem {
    prompt: String,
    negative_prompt: String,
    sampler_index: String,
    seed: i32,
    batch_size: u32,
    steps: u32,
    cfg_scale: u32,
    width: u32,
    height: u32,
    restore_faces: bool,
    send_images: bool,
    save_images: bool,
    alwayson_scripts: AlwaysonScripts,
}

#[derive(Debug, Deserialize, Serialize)]
struct AlwaysonScripts {
    ADetailer: ADetailerArgs,
}

#[derive(Debug, Deserialize, Serialize)]
struct ADetailerArgs {
    args: Vec<Args>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Args {
    ad_model: String,
    ad_prompt: String,
    ad_negative_prompt: String,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    images: Vec<String>,
    parameters: Value,
    info: String,
}

async fn send_request(api: &str, client: &Client, item: &DataItem) -> Result<ApiResponse, reqwest::Error> {
    let response = client.post(api).json(item).send().await?;
    let api_response: ApiResponse = response.json().await?;
    Ok(api_response)
}

async fn save_images(save_dir: &str, images: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(save_dir);
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }

    let current_datetime = Local::now().format("%Y%m%d%H%M%S");

    for (i, image) in images.into_iter().enumerate() {
        let data = base64::decode(&image)?;
        let filename = format!("{}_{}.png", current_datetime, i);
        let mut file = File::create(path.join(filename))?;
        file.write_all(&data)?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open("config.yml").expect("Unable to open the file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read the file");

    let config: Config = serde_yaml::from_str(&contents).expect("Unable to parse YAML");

    let client = Client::new();

    loop {
        for item in &config.Data {
            // println!("Sending request for item: {:?}", item);
            let api_response = send_request(&config.Api, &client, item).await?;
            // println!("Received response: {:?}", api_response);

            let now = Utc::now().with_timezone(&Shanghai).format("%Y-%m-%d %H:%M:%S");
            println!("【{}】 Saving images...", now);
            save_images(&config.SaveDir, api_response.images).await?;
            let end = Utc::now().with_timezone(&Shanghai).format("%Y-%m-%d %H:%M:%S");
            println!("【{}】 Images saved.", end);
        }

        if !config.RunningState {
            break;
        }

        sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}
