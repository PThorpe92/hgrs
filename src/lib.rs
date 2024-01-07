mod mercurial_file;

pub use crate::mercurial_file::FileStatus;
use crate::mercurial_file::MercurialFile;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct MercurialRepository<'a> {
    path: &'a Path,
    files: Vec<MercurialFile>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
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

/// Checks if the provided path is the root of a mercurial repository
pub fn is_root_mercurial_repository(path: &Path) -> bool {
    path.is_dir() && path.join(".hg").is_dir()
}

/// Checks if command exists and sets the proper environment variable to interact
/// with hg through a pipe. This function should be called first
pub fn check_install_init() -> Result<(), MercurialErr> {
    if matches!(Command::new("hg")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound,
    ) {
        return Err(MercurialErr::HgNotFound);
    } else if std::env::var("HGPLAIN").is_err() {
        std::env::set_var("HGPLAIN", "1");
    }
    Ok(())
}

/// Returns the closest parent directory that is a mercurial repository up to a maximum depth
/// of `depth_max`. If no repository is found, returns None.
pub fn find_parent_repo_recursively(path: &Path, depth_max: u32) -> Option<MercurialRepository> {
    if depth_max == 0 {
        None
    } else if is_root_mercurial_repository(path) {
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

fn get_repo_status(path: &Path) -> Result<Vec<MercurialFile>, MercurialErr> {
    Ok(String::from_utf8(
        Command::new("hg")
            .current_dir(path)
            .arg("status")
            .arg("--all")
            .output()
            .map_err(|_| MercurialErr::NotMercurialRepository)?
            .stdout,
    )
    .map_err(|_| MercurialErr::RepoWithError)?
    .lines()
    .map(MercurialFile::from)
    .collect())
}

impl<'a> MercurialRepository<'_> {
    pub fn new(path: &'a Path) -> Result<MercurialRepository<'a>, MercurialErr> {
        check_install_init()?;
        let repo = MercurialRepository {
            path,
            files: get_repo_status(path)?,
        };
        Ok(repo)
    }

    /// Updates all stored Repository info. Note that this does not update the actual files,
    /// only the state of the MercurialRepository struct, so it is not recommended
    /// to call this unless you believe the status has changed externally.
    pub fn update_repo(&mut self) -> Result<(), MercurialErr> {
        self.files = get_repo_status(self.path)?;
        Ok(())
    }

    /// Returns the status of a file relative to the repository, however not the root of the
    /// repository.
    pub fn get_file_status(&self, file_name: &'a Path) -> Result<FileStatus, MercurialErr> {
        if file_name.is_dir() {
            // For some reason, mercurial doesn't treat directories as tracked files. so I guess we do..
            // but this could be confusing where you
            return Ok(FileStatus::Directory);
        }
        let name = file_name
            .strip_prefix(self.path)
            .map_err(|_| MercurialErr::StatusError)?;
        match self.files.iter().find(|f| f.path() == name) {
            Some(f) => Ok(f.status()),
            None => Ok(FileStatus::NotTracked),
        }
    }

    /// Returns whether the repository has any files marked as anything other than clean
    pub fn is_dirty_repo(&self) -> bool {
        self.files.iter().any(|f| f.is_dirty())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_get_repo() {
        let repo = MercurialRepository::new(Path::new("./test_repo")).unwrap();
        // there are 8 files but the tests will fail occasionally cuz race condition and say its 9
        // because we create one later on in the tests and i'm lazy
        assert!(repo.files.len() >= 8);
        assert!(repo.files.len() < 10);
        assert!(repo.is_dirty_repo());
    }

    #[test]
    fn test_get_status() {
        let path = Path::new("./test_repo");
        assert!(is_root_mercurial_repository(path));
        let repo = MercurialRepository::new(path).unwrap();
        assert_eq!(
            repo.get_file_status(Path::new("not_a_file")),
            Err(MercurialErr::StatusError)
        );
        assert_eq!(
            repo.get_file_status(Path::new("./test_repo/file1"))
                .unwrap(),
            FileStatus::Clean
        );
        assert_eq!(
            repo.get_file_status(Path::new("./test_repo/deleted"))
                .unwrap(),
            FileStatus::Missing
        );
        assert_eq!(
            repo.get_file_status(Path::new("./test_repo/file2"))
                .unwrap(),
            FileStatus::Modified
        );
        assert_eq!(
            repo.get_file_status(Path::new("./test_repo/file4"))
                .unwrap(),
            FileStatus::Ignored
        );
        assert_eq!(
            repo.get_file_status(Path::new("./test_repo/directory"))
                .unwrap(),
            FileStatus::Directory
        );
        assert_eq!(
            repo.get_file_status(Path::new("./test_repo/directory/dirfile1"))
                .unwrap(),
            FileStatus::Clean
        );
    }
    #[test]
    fn test_find_parent_repo() {
        let repo = find_parent_repo_recursively(
            Path::new("./test_repo/directory/subdirectory/subdirectory2"),
            4,
        )
        .unwrap();
        assert!(repo.is_dirty_repo());
    }

    #[test]
    fn test_update_statuses() {
        let mut repo = MercurialRepository::new(Path::new("./test_repo")).unwrap();
        assert_eq!(repo.files.len(), 8);
        let _ = std::fs::File::create("./test_repo/file6").unwrap();
        repo.update_repo().unwrap();
        assert_eq!(
            repo.get_file_status(Path::new("./test_repo/file6"))
                .unwrap(),
            FileStatus::NotTracked
        );
        assert_eq!(repo.files.len(), 9);
        std::fs::remove_file("./test_repo/file6").unwrap();
    }

    #[test]
    fn test_is_mercurial_repository() {
        assert!(is_root_mercurial_repository(Path::new("./test_repo")));
        assert!(!is_root_mercurial_repository(Path::new(
            "./test_repo/directory"
        )));
    }

    #[test]
    fn test_check_hg_exists() {
        assert!(check_install_init().is_ok());
    }

    #[test]
    fn test_from_str() {
        let file = MercurialFile::from("A addfile");
        assert_eq!(file.status(), FileStatus::Added);
        assert_eq!(file.path(), Path::new("addfile"));
    }

    #[test]
    fn test_is_dirty_repo() {
        // should be dirty already due to test files
        let repo = MercurialRepository::new(Path::new("./test_repo")).unwrap();
        assert!(repo.is_dirty_repo());
    }
}
