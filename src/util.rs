use serde::{Serialize, Deserialize};
use requests::{self, ToJson};
use semver::Version;

#[derive(Eq, PartialEq)]
pub enum Category {
    Shared,
    Master,
    Server
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum VarReaderTypes {
    Bool(bool),
    I32(i32),
    I64(i64),
    F64(f64)
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum InDataTypes {
    Bool,
    I32,
    I64,
    F64
}

// Version, need_update
fn get_latest_version() -> Result<Version, ()> {
    let data = match requests::get("https://api.github.com/repos/sequal32/yourcontrol/releases/latest") {
        Ok(response) => match response.json() {
            Ok(data) => data,
            Err(_) => return Err(())
        }
        Err(_) => return Err(())
    };

    return match data["tag_name"].as_str() {
        Some(v) => match Version::parse(v) {
            Ok(v) => Ok(v),
            Err(_) => Err(())
        },
        None => Err(())
    };
}

fn get_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).unwrap()
}

pub fn app_get_versions() -> (Version, Option<Version>) {
    let app_ver = get_version();
    if let Ok(new_ver) = get_latest_version() {
        return (app_ver, Some(new_ver))
    }
    return (app_ver, None)
}