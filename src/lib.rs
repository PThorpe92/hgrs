mod mercurial_file;

pub use crate::mercurial_file::FileStatus;
use crate::mercurial_file::MercurialFile;
use log::debug;
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct MercurialRepository<'a> {
    path: &'a Path,
    files: Vec<MercurialFile>,
    pub raw_statuses: Vec<Cow<'a, str>>,
}

#[derive(Debug, Clone)]
pub enum MercurialErr {
    HgNotFound,
    NotMercurialRepository,
    RepoWithError,
    StatusError,
}

impl Display for MercurialErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MercurialErr::HgNotFound => write!(f, "Mercurial is not installed"),
            MercurialErr::NotMercurialRepository => write!(f, "Not a Mercurial repository"),
            MercurialErr::RepoWithError => write!(f, "Mercurial repository has errors"),
            MercurialErr::StatusError => write!(f, "Error getting file status"),
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
            Err(_) => {
                return match path.parent() {
                    Some(p) => find_parent_repo_recursively(p, depth_max - 1),
                    None => None,
                }
            }
        }
    } else {
        match path.parent() {
            Some(p) => find_parent_repo_recursively(p, depth_max - 1),
            None => return None,
        }
    }
}

impl<'a> MercurialRepository<'a> {
    pub fn check_hg_exists() -> bool {
        matches!(Command::new("which")
            .arg("hg")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status(),
            Ok(s) if s.success(),
        )
    }

    pub fn new(path_buf: &'a Path) -> Result<MercurialRepository<'a>, MercurialErr> {
        if !MercurialRepository::check_hg_exists() {
            return Err(MercurialErr::HgNotFound);
        }

        let raw_statuses = Command::new("hg")
            .current_dir(path_buf)
            .arg("status")
            .arg("--all")
            .output()
            .map_err(|_| MercurialErr::NotMercurialRepository)?
            .stdout;

        let statuses = String::from_utf8(raw_statuses).map_err(|_| MercurialErr::StatusError)?;
        let rows: Vec<_> = statuses
            .split('\n')
            .map(|s| Cow::Owned(s.to_string()))
            .collect();
        let mut repo = MercurialRepository {
            path: path_buf,
            files: vec![],
            raw_statuses: rows,
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
            .collect::<Vec<_>>();
    }

    pub fn update_repo(&mut self) -> Result<(), MercurialErr> {
        let raw_statuses = Command::new("hg")
            .current_dir(self.path)
            .arg("status")
            .arg("--all")
            .output()
            .map_err(|_| MercurialErr::NotMercurialRepository)?
            .stdout;
        let statuses = String::from_utf8(raw_statuses).map_err(|_| MercurialErr::StatusError)?;
        let rows: Vec<_> = statuses.split('\n').collect();
        self.raw_statuses = rows.iter().map(|s| Cow::Owned(s.to_string())).collect();
        self.set_files();
        Ok(())
    }

    pub fn get_status(&self, file_name: &'a Path) -> Result<FileStatus, MercurialErr> {
        let name = file_name.strip_prefix(self.path).unwrap_or(file_name);
        if file_name.is_dir() {
            debug!("{} is a directory", name.display());
            return Ok(FileStatus::Directory);
        }
        match self.files.iter().find(|f| f.path() == name) {
            Some(f) => Ok(f.status()),
            None => Ok(FileStatus::NotTracked),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_get_repo() {
        let repo = MercurialRepository::new(Path::new("./test_repo")).unwrap();
        assert_eq!(repo.files.len(), 8);
    }

    #[test]
    fn test_get_status() {
        let repo = MercurialRepository::new(Path::new("./test_repo")).unwrap();
        assert_eq!(
            repo.get_status(Path::new("./test_repo")).unwrap(),
            FileStatus::Directory
        );
        assert_eq!(
            repo.get_status(Path::new("./test_repo/addfile")).unwrap(),
            FileStatus::Added
        );
        assert_eq!(
            repo.get_status(Path::new("./test_repo/deleted")).unwrap(),
            FileStatus::Missing
        );
        assert_eq!(
            repo.get_status(Path::new("./test_repo/file2")).unwrap(),
            FileStatus::Modified
        );
        assert_eq!(
            repo.get_status(Path::new("./test_repo/file4")).unwrap(),
            FileStatus::Ignored
        );
        assert_eq!(
            repo.get_status(Path::new("./test_repo/directory")).unwrap(),
            FileStatus::Directory
        );
        assert_eq!(
            repo.get_status(Path::new("./test_repo/directory/dirfile1"))
                .unwrap(),
            FileStatus::Clean
        );
    }
    #[test]
    fn test_find_parent_repo() {
        let repo =
            find_parent_repo_recursively(Path::new("./test_repo/directory/dirfile1"), 3).unwrap();
        assert_eq!(repo.files.len(), 8);
    }

    #[test]
    fn test_update_statuses() {
        let mut repo = MercurialRepository::new(Path::new("./test_repo")).unwrap();
        assert_eq!(repo.files.len(), 8);
        let _ = std::fs::File::create("./test_repo/file6").unwrap();
        repo.update_repo().unwrap();
        assert_eq!(
            repo.get_status(Path::new("./test_repo/file6")).unwrap(),
            FileStatus::NotTracked
        );
        assert_eq!(repo.files.len(), 9);
        std::fs::remove_file("./test_repo/file6").unwrap();
    }

    #[test]
    fn test_is_mercurial_repository() {
        assert!(is_mercurial_repository(Path::new("./test_repo")));
        assert!(!is_mercurial_repository(Path::new("./test_repo/directory")));
    }

    #[test]
    fn test_check_hg_exists() {
        assert!(MercurialRepository::check_hg_exists());
    }

    #[test]
    fn test_from_str() {
        let file = MercurialFile::from("A addfile");
        assert_eq!(file.status(), FileStatus::Added);
        assert_eq!(file.path(), Path::new("addfile"));
    }
}
