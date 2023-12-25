use std::path::PathBuf;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FileStatus {
    Modified,
    Added,
    Removed,
    Clean,
    Missing,
    NotTracked,
    Ignored,
}

impl Default for FileStatus {
    fn default() -> Self {
        FileStatus::NotTracked
    }
}

#[derive(Debug, Clone)]
pub struct MercurialFile {
    pub path: PathBuf,
    pub status: FileStatus,
}

impl From<String> for MercurialFile {
    fn from(value: String) -> Self {
        let status = value.chars().nth(0).unwrap();
        let path: String = value.chars().skip(2).collect();

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
