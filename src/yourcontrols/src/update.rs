use attohttpc;
use semver::Version;
use serde_json::Value;
use std::env;
use std::{
    fs,
    io::{copy, Cursor},
};
use zip;

const RELEASE_DIRECT_URL: &str =
    "https://github.com/sequal32/yourcontrolsinstaller/releases/latest/download/installer.zip";
const PROGRAM_RELEASE_URL: &str =
    "https://api.github.com/repos/sequal32/yourcontrols/releases/latest";

const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:53.0) Gecko/20100101 Firefox/53.0";

pub enum DownloadInstallerError {
    RequestFailed(attohttpc::Error),
    MissingFieldJSON,
    IOError(std::io::Error),
    InvalidVersion(semver::SemVerError),
    ZipError(zip::result::ZipError),
}

impl std::fmt::Display for DownloadInstallerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadInstallerError::RequestFailed(e) => write!(f, "HTTP request failed: {}", e),
            DownloadInstallerError::MissingFieldJSON => write!(f, "Missing field in JSON"),
            DownloadInstallerError::IOError(e) => write!(f, "IO Error: {}", e),
            DownloadInstallerError::InvalidVersion(e) => write!(f, "Version Error: {}", e),
            DownloadInstallerError::ZipError(e) => write!(f, "Zip Error: {}", e),
        }
    }
}

pub struct Updater {
    latest_version: Option<Version>,
    latest_installer_bytes: Option<Vec<u8>>,
}

impl Updater {
    pub fn new() -> Self {
        Self {
            latest_version: None,
            latest_installer_bytes: None,
        }
    }

    fn get_url(&self, url: &str) -> Result<attohttpc::Response, attohttpc::Error> {
        attohttpc::get(url).header("User-Agent", USER_AGENT).send()
    }

    fn get_json_from_url(&self, url: &str) -> Result<Value, DownloadInstallerError> {
        // Download installer release info
        let response = match self.get_url(url) {
            Ok(response) => response,
            Err(e) => return Err(DownloadInstallerError::RequestFailed(e)),
        };

        match response.json() {
            Ok(data) => Ok(data),
            Err(e) => return Err(DownloadInstallerError::RequestFailed(e)),
        }
    }

    fn download_installer(&mut self) -> Result<&Vec<u8>, DownloadInstallerError> {
        // Download exe
        let response = match self.get_url(RELEASE_DIRECT_URL) {
            Ok(response) => response,
            Err(e) => return Err(DownloadInstallerError::RequestFailed(e)),
        };

        match response.bytes() {
            Ok(bytes) => {
                // Cache
                self.latest_installer_bytes = Some(bytes.clone());
                Ok(self.latest_installer_bytes.as_ref().unwrap())
            }
            Err(e) => Err(DownloadInstallerError::RequestFailed(e)),
        }
    }

    pub fn run_installer(&mut self) -> Result<(), DownloadInstallerError> {
        let installer_bytes = match self.latest_installer_bytes.as_ref() {
            Some(bytes) => bytes,
            None => self.download_installer()?,
        };

        let mut zip = match zip::ZipArchive::new(Cursor::new(installer_bytes)) {
            Ok(zip) => zip,
            Err(e) => return Err(DownloadInstallerError::ZipError(e)),
        };
        // Write files
        let mut dir = env::temp_dir();
        dir.push("YourControlsInstaller");
        fs::create_dir_all(dir.clone()).ok();

        let path = dir.to_str().unwrap();

        for file_index in 0..zip.len() {
            let mut file = zip.by_index(file_index).unwrap();

            match fs::File::create(format!("{}\\{}", path, file.name())) {
                Ok(mut file_handle) => copy(&mut file, &mut file_handle).ok(),
                Err(e) => return Err(DownloadInstallerError::IOError(e)),
            };
        }

        // Run exe
        dir.push("installer.exe");

        let mut process = std::process::Command::new(dir.as_os_str());
        process
            .stderr(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stdin(std::process::Stdio::null());

        match process.spawn() {
            Ok(_) => Ok(()),
            Err(e) => return Err(DownloadInstallerError::IOError(e)),
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
                }
                Err(e) => Err(DownloadInstallerError::InvalidVersion(e)),
            },
            None => Err(DownloadInstallerError::MissingFieldJSON),
        };
    }

    pub fn get_latest_version(&mut self) -> Result<&Version, DownloadInstallerError> {
        if self.latest_version.is_some() {
            return Ok(self.latest_version.as_ref().unwrap());
        } else {
            self.get_latest_version_info()
        }
    }

    pub fn get_version(&self) -> Version {
        Version::parse(env!("CARGO_PKG_VERSION")).unwrap()
    }
}
