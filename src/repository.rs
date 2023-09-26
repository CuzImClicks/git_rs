use core::panic;
use std::path::PathBuf;


pub struct Repository {
    worktree: PathBuf,
    gitdir: PathBuf,
    //conf: &'a Path
}

impl Repository {
    fn repo_path(&self, path: Vec<PathBuf>) -> PathBuf {
        let mut new_path = self.gitdir.clone();
        for p in path {
            new_path = new_path.join(p);
        }
        new_path
    }

    fn repo_file(&self, path: Vec<PathBuf>) -> PathBuf {

    }

    fn repo_dir(&self, path: Vec<PathBuf>, mkdir: bool) -> bool {
        let p = self.repo_path(path);

        if p.exists() {
            if p.is_dir() {
                return true;
            }

        }
        if mkdir {
            std::fs::create_dir_all(p).unwrap();
            return true;
        }
        return false
    }

    fn new(path: PathBuf, force: bool) -> Repository {
        let git_path: PathBuf = path.join(".git");
        let repo = Repository { worktree: path.clone(), gitdir: git_path };
        if !(force || repo.gitdir.is_dir()) {
            panic!("Not a git repository!");
        }

        repo
    }
}
