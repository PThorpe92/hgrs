use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum FileStatus {
    Modified,
    Added,
    Removed,
    Clean,
    Missing,
    #[default]
    NotTracked,
    Ignored,
    Directory,
}

#[derive(Debug, Clone)]
pub struct MercurialFile {
    path: PathBuf,
    status: FileStatus,
}

impl MercurialFile {
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn status(&self) -> FileStatus {
        self.status
    }
}

impl From<&str> for MercurialFile {
    fn from(value: &str) -> Self {
        let status = value.chars().nth(0).unwrap_or('?');
        let path: &str = value
            .get(2..)
            .unwrap_or_else(|| panic!("Invalid file: {}", value));

        MercurialFile {
            path: PathBuf::from(path),
            status: match status {
                'M' => FileStatus::Modified,
                'A' => FileStatus::Added,
                'R' => FileStatus::Removed,
                'C' => FileStatus::Clean,
                '!' => FileStatus::Missing,
                '?' => FileStatus::NotTracked,
                'I' => FileStatus::Ignored,
                _ => panic!("Unknown status: {}", status),
            },
        }
    }
}
