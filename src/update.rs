use bytes::Bytes;
use log::warn;
use {reqwest::{blocking::{ClientBuilder, Client}, header::{HeaderMap, HeaderValue}}};
use semver::Version;
use serde_json::Value;
use std::{io::copy, fs};
use std::env;

const INSTALLER_RELEASE_URL: &str = "https://api.github.com/repos/sequal32/yourcontrolsinstaller/releases/latest";
const PROGRAM_RELEASE_URL: &str = "https://api.github.com/repos/sequal32/yourcontrols/releases/latest";

pub enum DownloadInstallerError {
    RequestFailed(reqwest::Error),
    MissingFieldJSON,
    IOError(std::io::Error),
    InvalidVersion(semver::SemVerError)
}

impl std::fmt::Display for DownloadInstallerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadInstallerError::RequestFailed(e) => write!(f, "HTTP request failed: {}", e),
            DownloadInstallerError::MissingFieldJSON => write!(f, "Missing field in JSON"),
            DownloadInstallerError::IOError(e) => write!(f, "IO Error: {}", e),
            DownloadInstallerError::InvalidVersion(e) => write!(f, "Version Error: {}", e)
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
    client: Client,
    latest_version: Option<Version>,
    latest_installer_bytes: Option<Bytes>
}

impl Updater {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", HeaderValue::from_str("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:53.0) Gecko/20100101 Firefox/53.0").unwrap());

        Self {
            client: ClientBuilder::new().default_headers(headers).build().unwrap(),
            latest_version: None,
            latest_installer_bytes: None
        }
    }

    fn get_json_from_url(&self, url: &str) -> Result<Value, DownloadInstallerError> {
        // Download installer release info
        let response = match self.client.get(url).send() {
            Ok(response) => response,
            Err(e) => return Err(DownloadInstallerError::RequestFailed(e))
        };
        
        match response.json() {
            Ok(data) => Ok(data),
            Err(e) => return Err(DownloadInstallerError::RequestFailed(e))
        }
    }

    fn get_release_url(&self) -> Result<String, DownloadInstallerError> {
        let json = self.get_json_from_url(INSTALLER_RELEASE_URL)?;

        match get_url_from_json(&json) {
            Some(url) => Ok(url),
            None => {
                warn!("Missing field in JSON: {}", json);
                return Err(DownloadInstallerError::MissingFieldJSON)
            }
        }
    }

    fn download_installer(&mut self) -> Result<&Bytes, DownloadInstallerError> {
        // Download exe
        let response = match self.client.get(self.get_release_url()?.as_str()).send() {
            Ok(response) => response,
            Err(e) => return Err(DownloadInstallerError::RequestFailed(e))
        };

        match response.bytes() {
            Ok(bytes) => {
                // Cache
                self.latest_installer_bytes = Some(bytes.clone());
                Ok(self.latest_installer_bytes.as_ref().unwrap())
            },
            Err(e) => Err(DownloadInstallerError::RequestFailed(e))
        }
    }

    pub fn run_installer(&mut self) -> Result<(), DownloadInstallerError> {
        let installer_bytes = match self.latest_installer_bytes.as_ref() {
            Some(bytes) => bytes,
            None => self.download_installer()?
        };
        
        // Write exe
        let mut dir = env::temp_dir();
        dir.push("YourControlsInstaller.exe");

        let mut out = match fs::File::create(dir.clone()) {
            Ok(file) => file,
            Err(e) => return Err(DownloadInstallerError::IOError(e))
        };

        match copy(&mut installer_bytes.as_ref(), &mut out) {
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

    fn get_latest_version_info(&mut self) -> Result<&Version, DownloadInstallerError> {
        let json = self.get_json_from_url(PROGRAM_RELEASE_URL)?;
    
        return match json["tag_name"].as_str() {
            Some(v) => match Version::parse(v) {
                Ok(v) => {
                    // Cache
                    self.latest_version = Some(v);
                    Ok(self.latest_version.as_ref().unwrap())
                },
                Err(e) => Err(DownloadInstallerError::InvalidVersion(e))
            },
            None => Err(DownloadInstallerError::MissingFieldJSON)
        };
    }

    pub fn get_latest_version(&mut self) -> Result<&Version, DownloadInstallerError> {
        if self.latest_version.is_some() {
            return Ok(self.latest_version.as_ref().unwrap())
        } else {
            self.get_latest_version_info()
        }
    }
    
    pub fn get_version(&self) -> Version {
        Version::parse(env!("CARGO_PKG_VERSION")).unwrap()
    }
}
