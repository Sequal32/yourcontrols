use log::warn;
use {reqwest::{blocking::{ClientBuilder, Client}, header::{HeaderMap, HeaderValue}}};
use serde_json::Value;
use std::{io::copy, fs};
use std::env;

const INSTALLER_RELEASE_URL: &str = "https://api.github.com/repositories/311204382/releases/latest";

pub enum DownloadInstallerError {
    RequestFailed(reqwest::Error),
    MissingFieldJSON,
    IOError(std::io::Error)
}

impl std::fmt::Display for DownloadInstallerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadInstallerError::RequestFailed(e) => write!(f, "HTTP request failed: {}", e),
            DownloadInstallerError::MissingFieldJSON => write!(f, "Missing field in JSON"),
            DownloadInstallerError::IOError(e) => write!(f, "IO Error: {}", e)
        }
    }
}

fn get_url_from_json(data: &Value) -> Option<String> {
    Some(
        data["assets"]
        .as_array()?
        [0]
        .as_object()?
        ["browser_download_url"]
        .as_str()?
        .to_string()
    )
}

pub struct Updater {
    client: Client
}

impl Updater {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", HeaderValue::from_str("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:53.0) Gecko/20100101 Firefox/53.0").unwrap());

        Self {
            client: ClientBuilder::new().default_headers(headers).build().unwrap()
        }
    }

    fn get_release_url(&self) -> Result<String, DownloadInstallerError> {
        // Download installer release info
        let response = match self.client.get(INSTALLER_RELEASE_URL).send() {
            Ok(response) => response,
            Err(e) => return Err(DownloadInstallerError::RequestFailed(e))
        };
        
        let json_data: Value = match response.json() {
            Ok(data) => data,
            Err(e) => return Err(DownloadInstallerError::RequestFailed(e))
        };

        match get_url_from_json(&json_data) {
            Some(url) => Ok(url),
            None => {
                warn!("Missing field in JSON: {}", json_data);
                return Err(DownloadInstallerError::MissingFieldJSON)
            }
        }
    }

    pub fn download_and_run_installer(&self) -> Result<(), DownloadInstallerError> {
        // Download exe
        let response = match self.client.get(self.get_release_url()?.as_str()).send() {
            Ok(response) => response,
            Err(e) => return Err(DownloadInstallerError::RequestFailed(e))
        };

        let bytes = match response.bytes() {
            Ok(bytes) => bytes,
            Err(e) => return Err(DownloadInstallerError::RequestFailed(e))
        };
        // Write exe
        let mut dir = env::temp_dir();
        dir.push("YourControlsInstaller.exe");

        let mut out = match fs::File::create(dir.clone()) {
            Ok(file) => file,
            Err(e) => return Err(DownloadInstallerError::IOError(e))
        };

        match copy(&mut bytes.as_ref(), &mut out) {
            Ok(_) => {},
            Err(e) => return Err(DownloadInstallerError::IOError(e))
        };

        // Can't run file with an active file handle
        std::mem::drop(out);

        // Run exe
        let mut process = std::process::Command::new(dir.as_os_str());
        process
            .stderr(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stdin(std::process::Stdio::null());

        match process.spawn() {
            Ok(_) => Ok(()),
            Err(e) => return Err(DownloadInstallerError::IOError(e))
        }
    }
}
