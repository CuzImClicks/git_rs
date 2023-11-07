use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use anyhow::{anyhow, Context, Result};

use sha1::{Digest, Sha1};
use sha1::digest::FixedOutput;

use crate::repository::Repository;
use crate::utils::crlf_to_lf;

pub fn read_git_object(repo: &Repository, sha: &String) -> Result<Box<dyn GitObject>> {
    let path = repo.repo_git_path_vec(vec!["objects", &sha[0..2], &sha[2..]]);

    if !path.exists() {
        return Err(anyhow!("Object does not exist {}", sha));
    } else if !path.is_file() {
        return Err(anyhow!("Object isn't a file {}", sha));
    }

    let mut file = File::open(path).unwrap();
    let mut buf: Vec<u8> = vec![];
    file.read_to_end(&mut buf).unwrap();
    let raw: Vec<u8> = miniz_oxide::inflate::decompress_to_vec_zlib(&buf).unwrap();
    let x: usize = raw.iter().position(|x| *x == 32u8).unwrap();
    let fmt = String::from_utf8(raw[0..x].to_vec()).unwrap();

    let y = 1 + x + raw[x+1..].iter().position(|x| *x == 0u8).unwrap(); //+1 because \0 is 2 wide
    let size = String::from_utf8(raw[x+1..y].to_vec()).context("Failed to parse the size of the data")?.parse::<usize>().unwrap();
    if size != raw.len() -y - 1{
        return Err(anyhow!("Malformed object {}", sha));
    }
    deserialize(raw[y+1..].to_vec(), &fmt)
}


/// Creates a new GitObject from the serialized data of an object.
pub fn deserialize(data: Vec<u8>, fmt: &str) -> Result<Box<dyn GitObject>> {
    match fmt {
        "commit" => Ok(Box::new(GitCommit::new(data))),
        "blob" => Ok(Box::new(GitBlob::new(data))),
        "tree" => Ok(Box::new(GitTree::new(data))),
        "tag" => Ok(Box::new(GitTag::new(data))),
        _ => Err(anyhow!("Unknown type {}", fmt)),
    }
}


pub const OBJECT_TYPES: [&str; 4] = ["commit", "tree", "blob", "tag"];

pub trait GitObject {

    fn new(data: Vec<u8>) -> Self where Self: Sized;

    fn as_any(&self) -> &dyn std::any::Any;

    /// Writes the serialized form of the object to the repository object store, after
    /// compressing it with zlib deflate.
    fn write(&self, repo: &Repository) -> Result<(), String> {
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

    fn get_raw_data(&self) -> Vec<u8>;

    fn format(&self) -> &str;
}

pub struct GitCommit {
    raw_data: Vec<u8>,
    pub tree: String,
    pub parent: Vec<String>,
    pub author: String,
    pub committer: String,
    pub gpgsig: Option<String>,
    pub message: String
}

impl GitObject for GitCommit {
    fn new(data: Vec<u8>) -> Self where Self: Sized {
        let string: String = String::from_utf8(data.clone()).unwrap();
        let mut metadata: HashMap<&str, String> = HashMap::new();
        let big_split: Vec<&str> = string.splitn(2, "\n\n").collect::<Vec<&str>>();
        let message: String = big_split[1].to_string().trim_end().to_string();
        let header: Vec<&str> = big_split[0].split("\ngpgsig").collect::<Vec<&str>>();
        let gpgsig: Option<String> = if header.len() >= 2 { Some(header[1].to_string()) } else { None };
        for line in header[0].lines() {
            let split = line.splitn(2, ' ').collect::<Vec<&str>>();
            if split.len() == 1 {
                continue;
            }
            if split[0] == "parent" && metadata.contains_key("parent") {
                let parent: Vec<String> = vec![
                    metadata.get("parent").unwrap().to_string(),
                    split[1].to_string()
                ];
                metadata.insert(split[0], parent.join(" "));
                continue;
            }
            metadata.insert(split[0], split[1].to_string());
        }
        let tree: String = metadata.get("tree").unwrap_or(&String::new()).to_string();
        let parent: Vec<String> = metadata.get("parent")
            .unwrap_or(&String::new())
            .split(' ')
            .collect::<Vec<&str>>().
            iter()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let author: String = metadata.get("author").unwrap_or(&String::new()).to_string();
        let committer: String = metadata.get("committer").unwrap_or(&String::new()).to_string();

        GitCommit {
            raw_data: data,
            tree,
            parent,
            author,
            committer,
            gpgsig,
            message: message.to_string(),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_raw_data(&self) -> Vec<u8> {
        self.raw_data.clone()
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_raw_data(&self) -> Vec<u8> {
        self.raw_data.clone()
    }

    fn format(&self) -> &str {
        "blob"
    }
}

pub struct GitTree {
    raw_data: Vec<u8>,
}

pub struct GitTreeLeaf {
    pub mode: String,
    pub name: String,
    pub sha: String,
}

impl GitObject for GitTree {
    fn new(data: Vec<u8>) -> Self where Self: Sized {
        GitTree { raw_data: data }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_raw_data(&self) -> Vec<u8> {
        self.raw_data.clone()
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_raw_data(&self) -> Vec<u8> {
        self.raw_data.clone()
    }

    fn format(&self) -> &str {
        "tag"
    }
}
