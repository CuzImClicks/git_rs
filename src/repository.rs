use core::panic;
use std::{fs::File, io::Write, path::PathBuf};
use std::path::Path;

pub fn vec_to_pathbuf<T: AsRef<Path>>(paths: Vec<T>) -> PathBuf {
    let mut pathbuf = PathBuf::new();
    for path in paths {
        pathbuf.push(path);
    }
    pathbuf
}


pub fn find_repo<P: AsRef<Path>>(p: P) -> Option<PathBuf> {
    let p = p.as_ref();
    if p.join(".git").exists() {
        Some(p.to_path_buf())
    } else if let Some(parent) = p.parent() {
        find_repo(parent)
    } else {
        None
    }
}


#[derive(Debug)]
pub struct Repository {
    pub worktree: PathBuf,
    pub gitdir: PathBuf,
    config: configparser::ini::Ini,
    initialised: bool
}

impl Repository {

    pub fn repo_git_path_vec<T: AsRef<Path>>(&self, path: Vec<T>) -> PathBuf {
        let mut new_path = self.gitdir.clone();
        for p in path {
            new_path = new_path.join(p);
        }
        new_path
    }

    pub fn repo_git_path<T: AsRef<Path>>(&self, path: T) -> PathBuf {
        let new_path = self.gitdir.clone();
        new_path.join(path)
    }

    pub fn repo_path_vec<T: AsRef<Path>>(&self, path: Vec<T>) -> PathBuf {
        let mut new_path = self.worktree.clone();
        for p in path {
            new_path = new_path.join(p);
        }
        new_path
    }

    pub fn repo_path<T: AsRef<Path>>(&self, path: T) -> PathBuf {
        let new_path = self.worktree.clone();
        new_path.join(path)
    }

    pub fn repo_create_file<T: AsRef<Path>>(&self, path: T) -> Result<PathBuf, String> {
        let path = self.repo_git_path(path.as_ref());
        if path.exists() {
            return Ok(path);
        }

        let parent = if path.is_file() {
            path.parent().unwrap().to_path_buf()
        } else {
            path.clone()
        };

        if !parent.exists() {
            std::fs::create_dir_all(parent).unwrap();
        }

        if path.is_file() {
            File::create(path.clone()).unwrap();
        }
        Ok(path.clone())
    }

    pub fn repo_create_file_vec<T: AsRef<Path>>(&self, path: Vec<T>) -> Result<PathBuf, String> {
        let path = self.repo_git_path_vec(path);
        if path.exists() {
            return Ok(path);
        }

        let parent = if path.is_file() {
            path.parent().unwrap().to_path_buf()
        } else {
            path.clone()
        };

        if !parent.exists() {
            std::fs::create_dir_all(parent).unwrap();
        }

        if path.is_file() {
            File::create(path.clone()).unwrap();
        }
        Ok(path)
    }

    pub fn create(&mut self) -> Result<(), String> {
        if !self.worktree.exists() {
            std::fs::create_dir(self.worktree.clone()).expect("Failed to create directory");
        }
        self.repo_create_file("branches")?;
        self.repo_create_file("objects")?;
        self.repo_create_file_vec(vec!["refs", "tags"])?;
        self.repo_create_file_vec(vec!["refs", "heads"])?;
        
        let mut description = File::create(self.repo_git_path("description")).expect("Failed to create description file.");
        description.write_all("Unnamed repository; edit this file 'description' to name the repository.\n".as_bytes()).expect("Failed to write description file");
        
        let mut head = File::create(self.repo_git_path("HEAD")).unwrap();
        head.write_all("ref: refs/heads/master\n".as_bytes()).expect("Failed to write HEAD file.");

        let mut config = File::create(self.repo_git_path("config")).unwrap();
        config.write_all(self.default_config().writes().as_bytes()).expect("Failed to write config file.");
        self.initialised = true;
        Ok(())
    }

    fn read_config(&mut self) -> Result<(), String>{
        let cf = self.repo_git_path(PathBuf::from("config"));

        if cf.exists() && cf.is_file() {
            self.config.load(cf).unwrap();
            return Ok(());
        }
        Err(format!("Failed to read config file '{:?}'", cf))
    }

    fn default_config(&self) -> configparser::ini::Ini {
        let mut config = configparser::ini::Ini::new();
        config.set("core", "repositoryformatversion", Some("0".to_string()));
        config.set("core", "filemode", Some("false".to_string()));
        config.set("core", "bare", Some("false".to_string()));
        config
    }

    pub fn find_object(&self, name: String, format: &str, follow: bool) -> PathBuf {
        PathBuf::from(name)
    }

    pub fn new(path: PathBuf) -> Repository {
        let git_path: PathBuf = path.join(".git");
        let is_initialised = git_path.exists() && git_path.is_dir();
        let mut repo = Repository { worktree: path.clone(), gitdir: git_path.clone(), initialised: is_initialised, config: configparser::ini::Ini::new() };

        if repo.read_config().is_ok() {
             if let Ok(v) = repo.config.getint("core", "repositoryformatversion") {
                if v.unwrap() != 0 {
                    panic!("Unsupported repositoryformatversion '{:?}'", v);
                }
             }
        }

        repo
    }
}
