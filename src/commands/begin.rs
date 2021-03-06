use std::{fs, io::Write};

use crate::{paths::ProjectPaths, Error, Result, CONF_TEMPLATE};

pub struct Begin {
    pub paths: ProjectPaths,
    pub created_conf: bool,
    pub created_revisions: bool,
    pub created_root: bool,
}

impl Begin {
    /// Accepts a path string targeting a directory to set up project files:
    /// The directory will be created if it does not exist or will fail if
    /// pointing to an existing non-directory. This will then either verify
    /// that there is an empty `revisions` directory nested within it or
    /// create it if not already present. If any error occurs, any changes
    /// to the file system will be attempted to be reversed.
    pub fn new_project(p: &str) -> Result<Self> {
        let cmd = Self {
            paths: ProjectPaths::for_new_project(p)?,
            created_conf: false,
            created_revisions: false,
            created_root: false,
        };

        Ok(cmd.create_root()?.create_revisions()?.create_conf()?)
    }

    /// Attempts to create the project root directory if it doesn't exist,
    /// marking created as true if newly created.
    fn create_root(mut self) -> Result<Self> {
        if !self.paths.root.exists() {
            fs::create_dir(&self.paths.root)?;
            self.created_root = true;
        }

        Ok(self)
    }

    /// Attempts to create the revisions directory if it doesn't exist,
    /// marking created as true if newly created.
    fn create_revisions(mut self) -> Result<Self> {
        if !self.paths.revisions.exists() {
            if let Err(e) = fs::create_dir(&self.paths.revisions) {
                self.revert()?;
                return Err(Error::IoError(e));
            }

            self.created_revisions = true;
        }

        Ok(self)
    }

    /// Attempts to create the default configuration file. If a failure occurs,
    /// it will attempt to clean up any directory or file created during the command.
    fn create_conf(mut self) -> Result<Self> {
        let mut err = None;

        match fs::File::create(&self.paths.conf) {
            Ok(mut f) => {
                self.created_conf = true;

                if let Err(e) = f.write_all(CONF_TEMPLATE) {
                    err = Some(Error::IoError(e));
                }
            }
            Err(e) => {
                err = Some(Error::IoError(e));
            }
        }

        if let Some(e) = err {
            self.revert()?;

            return Err(e);
        }

        Ok(self)
    }

    /// Attempts to remove any directories or files created during command execution.
    fn revert(&self) -> Result<()> {
        if self.created_conf {
            fs::remove_file(&self.paths.conf)?;
        }

        if self.created_revisions {
            fs::remove_dir(&self.paths.revisions)?;
        }

        if self.created_root {
            fs::remove_dir(&self.paths.root)?;
        }

        Ok(())
    }
}
