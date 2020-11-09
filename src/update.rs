use log::{info,error};
use requests::{self, ToJson};
use std::fs;
use std::env;
use reqwest::{blocking::ClientBuilder, header::{HeaderMap, HeaderValue}};

pub fn run_installer() -> Result<(), SomeTypeOfError> {

    let mut headers = HeaderMap::new();
    headers.insert("User-Agent", HeaderValue::from_str("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:53.0) Gecko/20100101 Firefox/53.0").unwrap());
    
    let client = ClientBuilder::new().default_headers(headers).build().unwrap();

    let data = match requests::get("https://api.github.com/repositories/311204382/releases/latest") {
        Ok(response) => match response.json() {
            Ok(data) => data,
            Err(_e) => {
                error!("Something went wrong while trying to get update download link. \n {}", _e);
                return;
            },
        }
        Err(e) => {
            error!("Something went wrong while trying to get update download link. \n {}", e);
            return;
        }
    };
    let file_url = data["assets"][0]["browser_download_url"].as_str();
    let bytes = match client.get(file_url.unwrap_or("default string")).send() {
        Ok(response) => response.bytes().unwrap(),
        Err(e) => {
            error!("Something went wrong while trying to get download installer. \n {}", e);
            return;
        },
    };
    let mut dir = env::temp_dir();
    dir.push("Installer.exe");
    let mut out = fs::File::create(dir).expect("failed to create file");
    let mut slice: &[u8] = bytes.as_ref();
    copy(&mut slice, &mut out).expect("failed to copy content");
    return "Test";
}
