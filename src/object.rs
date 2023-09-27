use std::fs::File;
use std::io::{Read, Write};

use sha1::{Digest, Sha1};
use sha1::digest::FixedOutput;

use crate::repository::Repository;

pub fn read_git_object(repo: &Repository, sha: String) -> Result<Box<dyn GitObject>, String> {
    let path = repo.repo_git_path_vec(vec!["objects", &sha[0..2], &sha[2..]]);
    dbg!(&path);

    if !path.exists() {
        return Err(format!("Object does not exist {}", sha));
    } else if !path.is_file() {
        return Err(format!("Object isn't a file {}", sha));
    }

    let mut file = File::open(path).unwrap();
    let mut buf: Vec<u8> = vec![];
    file.read_to_end(&mut buf).unwrap();
    let raw: String = String::from_utf8(miniz_oxide::inflate::decompress_to_vec_zlib(&buf).unwrap()).unwrap();
    let x = raw.find(' ').unwrap();
    let fmt = raw[0..x].to_string();
    dbg!(&fmt);

    let y = 1 + x + raw[x+1..].find('\0').unwrap(); //+1 because \0 is 2 wide
    let size = raw[x+1..y].parse::<usize>().unwrap();
    if size != raw.len() -y - 1{
        return Err(format!("Malformed object {}", sha));
    }

    match &*fmt {
        "commit" => Ok(Box::new(GitCommit::new(raw[y+1..].as_bytes().to_vec()))),
        "blob" => Ok(Box::new(GitBlob::new(raw[y+1..].as_bytes().to_vec()))),
        "tree" => Ok(Box::new(GitTree::new(raw[y+1..].as_bytes().to_vec()))),
        "tag" => Ok(Box::new(GitTag::new(raw[y+1..].as_bytes().to_vec()))),
        _ => {
            Err(format!("Unknown type {}", fmt))
        }
    }
}

pub const OBJECT_TYPES: [&str; 4] = ["commit", "tree", "blob", "tag"];

pub trait GitObject {

    fn new(data: Vec<u8>) -> Self where Self: Sized;

    fn write(&self, repo: &Repository) {
        let data = self.serialize();
        let result = format!("{} {}\0{}", self.format(), &data.len(), String::from_utf8(data).unwrap());
        let mut hasher = Sha1::new();
        hasher.update(result.as_bytes());
        let sha = hasher.finalize_fixed().to_vec();
        let (before, after) = sha.split_at(2);
        let path = repo.repo_create_file_vec(vec!["objects", &*String::from_utf8(before.to_vec()).unwrap(), &*String::from_utf8(after.to_vec()).unwrap()]).unwrap();
        if !path.exists() {
            let mut f = File::create(path).unwrap();
            f.write_all(miniz_oxide::deflate::compress_to_vec_zlib(result.as_bytes(), 1).as_slice()).unwrap();
        }
    }

    fn init(&self) {

    }

    fn serialize(&self) -> Vec<u8>;

    fn deserialize(&self, data: Vec<u8>);
    
    fn format(&self) -> &str;
}

pub struct GitCommit {
    data: Vec<u8>,
}

impl GitObject for GitCommit {
    fn new(data: Vec<u8>) -> Self where Self: Sized {
        GitCommit { data }
    }

    fn serialize(&self) -> Vec<u8> {
        todo!()
    }

    fn deserialize(&self, data: Vec<u8>) {
        todo!()
    }

    fn format(&self) -> &str {
        "commit"
    }
}

pub struct GitBlob {
    data: Vec<u8>,
}

impl GitObject for GitBlob {
    fn new(data: Vec<u8>) -> Self where Self: Sized {
        GitBlob { data }
    }
    fn serialize(&self) -> Vec<u8> {
        self.data.clone()
    }
    fn deserialize(&self, data: Vec<u8>) {
        todo!()
    }
    fn format(&self) -> &str {
        "blob"
    }
}

pub struct GitTree {
    data: Vec<u8>,
}

impl GitObject for GitTree {
    fn new(data: Vec<u8>) -> Self where Self: Sized {
        GitTree { data }
    }

    fn serialize(&self) -> Vec<u8> {
        todo!()
    }

    fn deserialize(&self, data: Vec<u8>) {
        todo!()
    }

    fn format(&self) -> &str {
        "tree"
    }
}

pub struct GitTag {
    data: Vec<u8>,
}

impl GitObject for GitTag {
    fn new(data: Vec<u8>) -> Self where Self: Sized {
        GitTag { data }
    }

    fn serialize(&self) -> Vec<u8> {
        todo!()
    }

    fn deserialize(&self, data: Vec<u8>) {
        todo!()
    }

    fn format(&self) -> &str {
        "tag"
    }
}
