use serde::{Deserialize, Serialize};
use semver::Version;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
    TestNetwork {
        port: i64
    },
    UiReady,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct  Aircraft {
    newest_version: Option<Version>,
    installed_version: Option<Version>,
    install_locked: bool,
    name: String,
    author: String
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged, rename_all = "camelCase")]
pub enum UIEvents {
    StartUpText {
        text: String,
    },
    InitData {
        version: String,
        aircrafts: Vec<Aircraft>
    },
    LoadingComplete,
    NetworkTestResult {
        test: TestNetworkResult,
        status: ResultStatus,
    },
}
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum TestNetworkResult {
    CloudServer,
    CloudServerP2P,
    UPnP,
    Direct,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ResultStatus {
    Pending {},
    Error { reason: String },
    Success {},
}
