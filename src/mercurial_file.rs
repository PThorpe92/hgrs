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
    pub path: PathBuf,
    pub status: FileStatus,
}

impl From<&str> for MercurialFile {
    fn from(value: &str) -> Self {
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
