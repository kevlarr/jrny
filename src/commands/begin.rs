use std::{fs, io::Write, path::PathBuf};

use log::info;

use crate::{
    CONF_TEMPLATE,
    ENV,
    ENV_TEMPLATE,
    ENV_EX,
    ENV_EX_TEMPLATE,
    Result,
    project::ProjectPaths,
};

/// Sets up relevant files and directories for a new revision timeline
#[derive(Debug)]
pub struct Begin {
    // TODO This has gotten gross
    paths: ProjectPaths,
    created_revisions: bool,
    created_root: bool,
    created_conf: bool,
    created_env: bool,
    created_env_ex: bool,
}

impl Begin {
    /// Accepts a path string targeting a directory to set up project files:
    /// The directory will be created if it does not exist or will fail if
    /// pointing to an existing non-directory. This will then either verify
    /// that there is an empty `revisions` directory nested within it or
    /// create it if not already present. If any error occurs, any changes
    /// to the file system will be attempted to be reversed.
    pub fn execute(dirpath: &PathBuf) -> Result<()> {
        let paths = ProjectPaths::for_new_project(&dirpath)?;

        let mut cmd = Self {
            paths,
            created_conf: false,
            created_env: false,
            created_env_ex: false,
            created_revisions: false,
            created_root: false,
        };

        cmd.create_root()
            .and_then(|_| cmd.create_revisions())
            .and_then(|_| cmd.create_conf())
            .and_then(|_| cmd.create_env())
            .map_err(|e| match cmd.revert() {
                Ok(_) => e,
                // TODO there should really be better error handling, since this
                // will lose the original error
                Err(err) => err,
            })?;

        info!("A journey has begun");

        print_path("  ",     &cmd.paths.root_dir,      cmd.created_root);
        print_path("  ├── ", &cmd.paths.revisions_dir, cmd.created_revisions);
        print_path("  └── ", &cmd.paths.conf_file,     cmd.created_conf);
        print_path("  └── ", &cmd.paths.env_file,      cmd.created_env);
        print_path("  └── ", &cmd.paths.env_ex_file,   cmd.created_env_ex);

        Ok(())
    }

    /// Attempts to create the project root directory if it doesn't exist.
    fn create_root(&mut self) -> Result<()> {
        if !self.paths.root_dir.exists() {
            fs::create_dir(&self.paths.root_dir)?;
            self.created_root = true;
        }

        Ok(())
    }

    /// Attempts to create the revisions directory if it doesn't exist.
    fn create_revisions(&mut self) -> Result<()> {
        if !self.paths.revisions_dir.exists() {
            fs::create_dir(&self.paths.revisions_dir)?;
            self.created_revisions = true;
        }

        Ok(())
    }

    /// Attempts to create the default configuration file.
    fn create_conf(&mut self) -> Result<()> {
        // This will truncate the file if it exists, so it relies on
        // the paths having already checked that the file isn't present.
        //
        // TODO Clean up that logic.
        let mut f = fs::File::create(&self.paths.conf_file)?;
        self.created_conf = true;
        f.write_all(CONF_TEMPLATE.as_bytes())?;

        Ok(())
    }

    /// Attempts to create the default environment file & example file.
    fn create_env(&mut self) -> Result<()> {
        let mut f = fs::File::create(&self.paths.env_file)?;
        self.created_env = true;
        f.write_all(ENV_TEMPLATE.as_bytes())?;

        let mut f = fs::File::create(&self.paths.env_ex_file)?;
        self.created_env_ex = true;
        f.write_all(ENV_EX_TEMPLATE.as_bytes())?;

        Ok(())
    }

    /// Attempts to remove any directories or files created during command execution.
    fn revert(&self) -> Result<()> {
        if self.created_conf {
            fs::remove_file(&self.paths.conf_file)?;
        }

        if self.created_env {
            fs::remove_file(&self.paths.env_file)?;
        }

        if self.created_env_ex {
            fs::remove_file(&self.paths.env_ex_file)?;
        }

        if self.created_revisions {
            fs::remove_dir(&self.paths.revisions_dir)?;
        }

        if self.created_root {
            fs::remove_dir(&self.paths.root_dir)?;
        }

        Ok(())
    }
}

/// Prints path string with optional prefix and "[created]" suffix if the created
/// condition is true.
fn print_path(prefix: &str, path: &PathBuf, created: bool) {
    info!(
        "{}{}{}",
        prefix,
        path.display(),
        if created { " [created]" } else { "" },
    );
}
