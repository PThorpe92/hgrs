mod mercurial_file;

pub use crate::mercurial_file::FileStatus;
use crate::mercurial_file::MercurialFile;
use log::debug;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct MercurialRepository {
    fail: bool,
    path: PathBuf,
    files: Vec<MercurialFile>,
    pub raw_statuses: Vec<String>,
}

pub fn is_mercurial_repository(path: &PathBuf) -> bool {
    if !path.exists() {
        return false;
    }
    if !path.is_dir() {
        return false;
    }
    if !path.join(".hg").exists() {
        return false;
    }
    true
}

pub fn find_repo_recursively(path: &PathBuf, mut depth_max: u32) -> Option<MercurialRepository> {
    if is_mercurial_repository(path) {
        return Some(MercurialRepository::new(path));
    }
    let mut p = path.clone();
    while depth_max > 0 {
        p = p.parent().unwrap().to_path_buf();
        if is_mercurial_repository(&p) {
            return Some(MercurialRepository::new(&p));
        }
        depth_max -= 1;
    }
    None
}

impl MercurialRepository {
    pub fn new(path_buf: &PathBuf) -> MercurialRepository {
        let fail = match Command::new("which")
            .arg("hg")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
        {
            Ok(s) if s.success() => false,
            _ => true,
        };
        if fail {
            panic!("hg not found");
        };
        let raw_statuses = Command::new("hg")
            .current_dir(path_buf)
            .arg("status")
            .arg("--all")
            .output()
            .unwrap()
            .stdout;
        let statuses = String::from_utf8(raw_statuses).unwrap();
        let rows: Vec<_> = statuses.split('\n').collect();
        let mut repo = MercurialRepository {
            fail,
            path: (*(path_buf.clone())).to_owned(),
            files: vec![],
            raw_statuses: rows.iter().map(|s| s.to_string()).collect(),
        };
        repo.set_files();
        repo
    }

    fn set_files(&mut self) {
        self.files = self
            .raw_statuses
            .clone()
            .iter()
            .filter(|s| !s.is_empty())
            .map(|r| MercurialFile::from(r.to_string()))
            .collect();
    }

    pub fn update_statuses(&mut self) {
        let raw_statuses = Command::new("hg")
            .current_dir(&self.path)
            .arg("status")
            .arg("--all")
            .output()
            .unwrap()
            .stdout;
        let statuses = String::from_utf8(raw_statuses).unwrap();
        let rows: Vec<_> = statuses.split('\n').collect();
        self.raw_statuses = rows.iter().map(|s| s.to_string()).collect();
        self.set_files()
    }

    pub fn get_status(&self, file_name: &PathBuf) -> FileStatus {
        if self.fail {
            panic!("hg not found");
        }
        let name = file_name.strip_prefix(&self.path).unwrap();
        if file_name.is_dir() {
            debug!("{} is a directory", name.display());
            return FileStatus::Directory;
        }
        self.files
            .iter()
            .find(|f| f.path == name)
            .unwrap()
            .status
            .clone()
    }
}
