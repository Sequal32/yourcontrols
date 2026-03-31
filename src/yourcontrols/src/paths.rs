use std::{fs::read_dir, io, path::PathBuf};

/// Helper for resolving definition paths based on sim and config, and getting available configs for sims.
pub struct DefinitionPathResolver;

impl DefinitionPathResolver {
    /// Gets the path to the definition file for the given sim and config, if it exists.
    pub fn from_sim_and_config(sim: &str, config: &str) -> Option<PathBuf> {
        let path = PathBuf::from(format!("definitions/{}/aircraft/{}", sim, config));

        if path.is_file() {
            Some(path)
        } else {
            None
        }
    }

    /// Gets the paths to all definition files for the given sim.
    pub fn get_filenames(sim: &str) -> io::Result<Vec<String>> {
        let mut filenames = Vec::new();
        let dir_path = format!("definitions/{}/aircraft/", sim);

        for file in read_dir(dir_path)? {
            let file = file?;
            filenames.push(
                file.path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            )
        }

        Ok(filenames)
    }

    pub fn get_fs_2020_configs() -> io::Result<Vec<String>> {
        Self::get_filenames("FS2020")
    }

    pub fn get_fs_2024_configs() -> io::Result<Vec<String>> {
        Self::get_filenames("FS2024")
    }
}
