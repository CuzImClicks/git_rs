use core::panic;
use std::{path::PathBuf, fs::File, io::Write};


pub struct Repository {
    worktree: PathBuf,
    gitdir: PathBuf,
    initialised: bool
}

impl Repository {
    fn repo_path(&self, path: Vec<PathBuf>) -> PathBuf {
        let mut new_path = self.gitdir.clone();
        for p in path {
            new_path = new_path.join(p);
        }
        new_path
    }

    fn repo_file(&self, path: Vec<PathBuf>, mkdir: bool) -> Result<PathBuf, String> {
        if let Ok(_) = match mkdir {
            true => self.repo_dir_create(path.split_last().unwrap().1.to_vec()),
            false => self.repo_dir(path.split_last().unwrap().1.to_vec())
        } {
            return Ok(self.repo_path(path));
        }
        Err(format!("{:?}", path))
    }

    fn repo_dir(&self, path: Vec<PathBuf>) -> Result<PathBuf, String> {
        let p = self.repo_path(path);

        if p.exists() || p.is_dir() {
            return Ok(p);
        }
        return Err(format!("'{:?}' is not a directory", p).to_string());
    }

    fn repo_dir_create(&self, path: Vec<PathBuf>) -> Result<PathBuf, String> {
        let p = self.repo_path(path);

        if p.exists() {
            if p.is_dir() {
                return Ok(p);
            }
            return Err(format!("'{:?}' is not a directory", p).to_string());
        }
        std::fs::create_dir_all(p.clone()).unwrap();
        return Ok(p);
    }

    fn create(&self) -> Result<(), String> {
        if self.worktree.exists() {
            if self.worktree.read_dir().unwrap().count() != 0 {
                return Err("Directory is not empty!".to_string());
            }
        } else {
            std::fs::create_dir(self.worktree.clone()).expect("Failed to create directory");
        }
        self.repo_dir_create(vec![PathBuf::from("branches")])?;
        self.repo_dir_create(vec![PathBuf::from("objects")])?;
        self.repo_dir_create(vec![PathBuf::from("refs"), PathBuf::from("tags")])?;
        self.repo_dir_create(vec![PathBuf::from("refs"), PathBuf::from("heads")])?;
        
        let mut description = File::create(self.repo_file(vec![PathBuf::from("description")], false)?).expect("Failed to create description file.");
        let _ = description.write_all("Unnamed repository; edit this file 'description' to name the repository.\n".as_bytes()).expect("Failed to write description file");
        
        let mut head = File::create(self.repo_file(vec![PathBuf::from("HEAD")], false)?).expect("Failed to create HEAD file.");
        let _ = head.write_all("ref: refs/heads/master\n".as_bytes()).expect("Failed to write HEAD file.");

        Ok(())


    }

    fn new(path: PathBuf) -> Repository {
        let git_path: PathBuf = path.join(".git");
        if !path.is_file() {
            panic!("Repository is a file!");
        }
        let repo = Repository { worktree: path.clone(), gitdir: git_path.clone(), initialised: git_path.is_dir() };
        

        repo
    }
}
