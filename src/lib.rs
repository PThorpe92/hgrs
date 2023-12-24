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
}

impl MercurialRepository {
    pub fn new(path_buf: PathBuf) -> MercurialRepository {
        let mut fail = false;
        match Command::new("which").arg("hg").status() {
            Ok(s) if s.success() => fail = false,
            _ => fail = true,
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
        let files: Vec<MercurialFile> = rows
            .iter()
            .filter(|s| !s.is_empty())
            .map(|r| MercurialFile::from(r.to_string()))
            .collect();
        MercurialRepository {
            fail,
            path: path_buf,
            files,
        }
    }

    pub fn get_status(&self, file_name: PathBuf) -> FileStatus {
        if self.fail {
            panic!("hg not found");
        }
        self.files
            .iter()
            .find(|f| f.path == file_name)
            .unwrap()
            .status
            .clone()
    }
}
