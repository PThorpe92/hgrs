mod mercurial_file;

pub use crate::mercurial_file::FileStatus;
use crate::mercurial_file::MercurialFile;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub struct MercurialRepository {
    fail: bool,
    path: PathBuf,
    files: Vec<MercurialFile>,
    pub raw_statuses: Vec<String>,
}

impl MercurialRepository {
    pub fn new(path_buf: PathBuf) -> MercurialRepository {
        let fail = match Command::new("which").arg("hg").status() {
            Ok(s) if s.success() => false,
            _ => true,
        };
        if fail {
            panic!("hg not found");
        };
        let raw_statuses = Command::new("hg")
            .current_dir(&path_buf)
            .arg("status")
            .arg("--all")
            .output()
            .unwrap()
            .stdout;
        let statuses = String::from_utf8(raw_statuses).unwrap();
        let rows: Vec<_> = statuses.split('\n').collect();
        let mut repo = MercurialRepository {
            fail,
            path: path_buf,
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
        self.files
            .iter()
            .find(|f| f.path == name)
            .unwrap()
            .status
            .clone()
    }
}
