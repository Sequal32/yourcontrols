use crate::ui::AircraftInstallData;
use anyhow::Result;
use attohttpc;
use semver::Version;
use serde::Deserialize;
use serde_yaml;
use std::collections::HashMap;
use std::fs::{read_dir, File};
use std::path::PathBuf;

pub const UPDATE_URL: &'static str =
    "https://www.github.com/sequal32/yourcontrols/definitions/aircraft/info.json";
pub const AIRCRAFT_PATH: &'static str = "definitions/aircraft";

type AircraftInfoMap = HashMap<String, AircraftInfo>;
type AircraftInstallMap = HashMap<String, AircraftInstallData>;

#[derive(Deserialize, Clone)]
struct AircraftInfo {
    pub name: String,
    pub author: String,
    pub version: Version,
    pub url: Option<String>,
    pub file_name: String,
}

pub struct DefinitionsUpdater {
    update_info: AircraftInfoMap,
    database: AircraftInstallMap,
}

impl DefinitionsUpdater {
    pub fn new() -> Self {
        Self {
            update_info: AircraftInfoMap::new(),
            database: AircraftInstallMap::new(),
        }
    }

    fn fetch_aircraft_list() -> Result<AircraftInfoMap> {
        let aircraft: Vec<AircraftInfo> =
            serde_json::from_value(attohttpc::get(UPDATE_URL).send()?.json()?)?;

        Ok(aircraft.into_iter().map(|x| (x.name.clone(), x)).collect())
    }

    fn detect_installed_aircraft() -> Result<Vec<AircraftInfo>> {
        let mut installed = Vec::new();

        for dir in read_dir("definitions/aircraft")? {
            let dir = dir?;

            let info: AircraftInfo = serde_yaml::from_reader(File::open(dir.path())?)?;
            installed.push(info);
        }

        return Ok(installed);
    }

    fn update_aircraft_info(&mut self, new_info: AircraftInfo, is_newest: bool) {
        if self.database.get(&new_info.name).is_none() {
            self.database
                .insert(new_info.name.clone(), Default::default());
        }

        let info = self.database.get_mut(&new_info.name).unwrap();

        info.author = new_info.author;
        info.name = new_info.name;
        if is_newest {
            info.newest_version = Some(new_info.version);
        } else {
            info.installed_version = Some(new_info.version)
        }
    }

    pub fn fetch_data(&mut self) -> Result<()> {
        self.update_info = Self::fetch_aircraft_list()?;

        for (_, info) in self.update_info.clone() {
            self.update_aircraft_info(info, true);
        }

        self.scan_installed()?;

        Ok(())
    }

    pub fn scan_installed(&mut self) -> Result<()> {
        for info in Self::detect_installed_aircraft()? {
            self.update_aircraft_info(info, false);
        }

        Ok(())
    }

    pub fn get_all_aircraft_info(&self) -> Vec<AircraftInstallData> {
        self.database.values().into_iter().cloned().collect()
    }

    pub fn update_aircraft(&self, aircraft: Vec<String>) -> Result<()> {
        for aircraft_to_update in aircraft {
            let update_info = match self.update_info.get(&aircraft_to_update) {
                Some(i) => i,
                None => continue,
            };

            let url = match &update_info.url {
                Some(u) => u,
                None => continue,
            };

            let response = attohttpc::get(url).send()?;
            let mut path = PathBuf::from(AIRCRAFT_PATH);
            path.push(&update_info.file_name);

            response.write_to(File::open(path)?)?;
        }

        Ok(())
        // TODO: Handle community folder installation
    }

    pub fn update_core_files(&self) {}
}
