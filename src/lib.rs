mod mercurial_file;

pub use crate::mercurial_file::FileStatus;
use crate::mercurial_file::MercurialFile;
use log::debug;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct MercurialRepository<'a> {
    path: &'a Path,
    files: Vec<MercurialFile>,
    // tried to figure out how not to own the strings here but couldn't do it
    pub raw_statuses: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum MercurialErr {
    HgNotFound,
    NotMercurialRepository,
    RepoWithError,
}
impl Display for MercurialErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MercurialErr::HgNotFound => write!(f, "Mercurial is not installed"),
            MercurialErr::NotMercurialRepository => write!(f, "Not a Mercurial repository"),
            MercurialErr::RepoWithError => write!(f, "Mercurial repository has errors"),
        }
    }
}
impl std::error::Error for MercurialErr {}

pub fn is_mercurial_repository(path: &Path) -> bool {
    path.join(".hg").exists()
}

pub fn find_parent_repo_recursively(path: &Path, depth_max: u32) -> Option<MercurialRepository> {
    if depth_max == 0 {
        None
    } else if is_mercurial_repository(path) {
        match MercurialRepository::new(path) {
            Ok(repo) => return Some(repo),
            Err(_) => return find_parent_repo_recursively(path.parent().unwrap(), depth_max - 1),
        }
    } else {
        return find_parent_repo_recursively(path.parent().unwrap(), depth_max - 1);
    }
}

fn check_hg_exists() -> bool {
    matches!(Command::new("which")
        .arg("hg")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status(),
        Ok(s) if s.success(),
    )
}

impl<'a> MercurialRepository<'a> {
    pub fn new(path_buf: &'a Path) -> Result<MercurialRepository, MercurialErr> {
        if !check_hg_exists() {
            return Err(MercurialErr::HgNotFound);
        }
        let raw_statuses = Command::new("hg")
            .current_dir(path_buf)
            .arg("status")
            .arg("--all")
            .output()
            .map_err(|_| MercurialErr::NotMercurialRepository)?
            .stdout;

        let statuses = String::from_utf8(raw_statuses).unwrap();
        let rows: Vec<_> = statuses.split('\n').collect();
        let mut repo = MercurialRepository {
            path: path_buf,
            files: vec![],
            raw_statuses: rows.iter().map(|s| s.to_string()).collect(),
        };
        repo.set_files();
        Ok(repo)
    }

    fn set_files(&mut self) {
        self.files = self
            .raw_statuses
            .iter()
            .by_ref()
            .filter(|s| !s.is_empty())
            .map(|r| MercurialFile::from(r.as_ref()))
            .collect();
    }

    pub fn update_statuses(&mut self) -> Result<(), MercurialErr> {
        let raw_statuses = Command::new("hg")
            .current_dir(self.path)
            .arg("status")
            .arg("--all")
            .output()
            .map_err(|_| MercurialErr::NotMercurialRepository)?
            .stdout;
        let statuses = String::from_utf8(raw_statuses).unwrap();
        let rows: Vec<_> = statuses.split('\n').collect();
        self.raw_statuses = rows.iter().map(|s| s.to_string()).collect();
        self.set_files();
        Ok(())
    }

    pub fn get_status(&self, file_name: &'a Path) -> Result<FileStatus, MercurialErr> {
        let name = file_name.strip_prefix(self.path).unwrap();
        if file_name.is_dir() {
            debug!("{} is a directory", name.display());
            return Ok(FileStatus::Directory);
        }
        match self.files.iter().find(|f| f.path.as_path() == name) {
            Some(f) => Ok(f.status),
            None => Ok(FileStatus::NotTracked),
        }
    }
}
