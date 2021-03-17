use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
    TestNetwork {
        port: i64
    },
    UiReady,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged, rename_all = "camelCase")]
pub enum UIEvents {
    StartUpText {
        text: String,
    },
    InitData {
        version: String,
        acupdate: bool,
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
