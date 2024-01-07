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
impl FileStatus {
    pub fn is_dirty(&self) -> bool {
        !matches!(self, FileStatus::Clean,)
    }
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
    pub fn is_dirty(&self) -> bool {
        self.status.is_dirty()
    }
}

impl From<char> for FileStatus {
    fn from(value: char) -> Self {
        match value {
            'M' => FileStatus::Modified,
            'A' => FileStatus::Added,
            'R' => FileStatus::Removed,
            'C' => FileStatus::Clean,
            '!' => FileStatus::Missing,
            '?' => FileStatus::NotTracked,
            'I' => FileStatus::Ignored,
            _ => panic!("Unknown status: {}", value),
        }
    }
}

impl From<&str> for MercurialFile {
    fn from(value: &str) -> Self {
        let status = value.chars().nth(0).unwrap_or('?');
        let path: &str = value.split_ascii_whitespace().nth(1).unwrap_or("");

        MercurialFile {
            path: PathBuf::from(path),
            status: status.into(),
        }
    }
}
