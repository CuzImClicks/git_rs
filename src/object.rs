use std::fs::File;
use std::io::{Read, Write};

use sha1::{Digest, Sha1};
use sha1::digest::FixedOutput;

use crate::repository::Repository;
use crate::utils::crlf_to_lf;

pub fn read_git_object(repo: &Repository, sha: String) -> Result<Box<dyn GitObject>, String> {
    let path = repo.repo_git_path_vec(vec!["objects", &sha[0..2], &sha[2..]]);

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

    let y = 1 + x + raw[x+1..].find('\0').unwrap(); //+1 because \0 is 2 wide
    let size = raw[x+1..y].parse::<usize>().unwrap();
    if size != raw.len() -y - 1{
        return Err(format!("Malformed object {}", sha));
    }
    deserialize(raw[y+1..].as_bytes().to_vec(), &fmt)
}


/// Creates a new GitObject from the serialized data of an object.
pub fn deserialize(data: Vec<u8>, fmt: &str) -> Result<Box<dyn GitObject>, String> {
    match fmt {
        "commit" => Ok(Box::new(GitCommit::new(data))),
        "blob" => Ok(Box::new(GitBlob::new(data))),
        "tree" => Ok(Box::new(GitTree::new(data))),
        "tag" => Ok(Box::new(GitTag::new(data))),
        _ => Err(format!("Unknown type {}", fmt)),
    }
}


pub const OBJECT_TYPES: [&str; 4] = ["commit", "tree", "blob", "tag"];

pub trait GitObject {

    fn new(data: Vec<u8>) -> Self where Self: Sized;

    /// Writes the serialized form of the object to the repository object store, after
    /// compressing it with zlib deflate.
    fn write(&self, repo: &Repository) -> Result<(), String>{
        let data: Vec<u8> = self.get_raw_data();
        let (before, after) = &data.split_at(2);
        let path = repo.repo_create_file_vec(vec!["objects", &*String::from_utf8(before.to_vec()).unwrap(), &*String::from_utf8(after.to_vec()).unwrap()]).unwrap();
        let result = self.serialize();
        if !path.exists() {
            let mut f = File::create(path).unwrap();
            f.write_all(miniz_oxide::deflate::compress_to_vec_zlib(result.as_bytes(), 1).as_slice()).unwrap();
            Ok(())
        } else {
            Err("Object already exists".to_string())
        }
    }

    /// Returns a hash of the serialized data.
    ///
    /// See: [`GitObject::serialize`]
    fn hash(&self) -> String {
        let mut hasher = Sha1::new();
        hasher.update(self.serialize().as_bytes());
        let finalized = hasher.finalize_fixed();
        hex::encode(finalized)
    }

    /// Returns the object in serialized form.
    ///
    /// `<blob|tree|commit|tag> <len_data>\0<data>`
    ///
    /// See: [`GitObject::get_raw_data`] is the data of the object.
    fn serialize(&self) -> String {
        let data = self.get_raw_data();
        let result = format!("{} {}\0{}", self.format(), &data.len(), String::from_utf8(data.clone()).unwrap());
        result
    }


    fn init(&self) {

    }

    fn get_raw_data(&self) -> Vec<u8>;

    fn deserialize(&self, data: Vec<u8>);
    
    fn format(&self) -> &str;
}

pub struct GitCommit {
    raw_data: Vec<u8>,
}

impl GitObject for GitCommit {
    fn new(data: Vec<u8>) -> Self where Self: Sized {
        GitCommit { raw_data: data }
    }

    fn get_raw_data(&self) -> Vec<u8> {
        self.raw_data.clone()
    }

    fn deserialize(&self, data: Vec<u8>) {
        todo!()
    }

    fn format(&self) -> &str {
        "commit"
    }
}

pub struct GitBlob {
    raw_data: Vec<u8>,
}

impl GitObject for GitBlob {
    fn new(data: Vec<u8>) -> Self where Self: Sized {
        GitBlob { raw_data: crlf_to_lf(&data) }
    }
    fn get_raw_data(&self) -> Vec<u8> {
        self.raw_data.clone()
    }
    fn deserialize(&self, data: Vec<u8>) {
        todo!()
    }
    fn format(&self) -> &str {
        "blob"
    }
}

pub struct GitTree {
    raw_data: Vec<u8>,
}

impl GitObject for GitTree {
    fn new(data: Vec<u8>) -> Self where Self: Sized {
        GitTree { raw_data: data }
    }

    fn get_raw_data(&self) -> Vec<u8> {
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
    raw_data: Vec<u8>,
}

impl GitObject for GitTag {
    fn new(data: Vec<u8>) -> Self where Self: Sized {
        GitTag { raw_data: data }
    }

    fn get_raw_data(&self) -> Vec<u8> {
        self.raw_data.clone()
    }

    fn deserialize(&self, data: Vec<u8>) {
        todo!()
    }

    fn format(&self) -> &str {
        "tag"
    }
}
