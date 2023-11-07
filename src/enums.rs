use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::{Read, Write};
use crate::repository::Repository;
use anyhow::{anyhow, Context, Result};
use sha1::{Sha1, Digest};
use sha1::digest::FixedOutput;

#[derive(Debug)]
pub enum GitObject {
    Blob { raw_data: Vec<u8> },
    Commit { 
        raw_data: Vec<u8>, 
        tree: String, 
        parent: Vec<String>, 
        author: String,
        committer: String,
        gpgsig: Option<String>,
        message: String 
    },
    Tree {
        raw_data: Vec<u8>
    },
    Tag {
        raw_data: Vec<u8>
    }
}

pub fn get_git_type(object: &GitObject) -> &'static str {
    match object {
        GitObject::Blob { .. } => "blob",
        GitObject::Commit { .. } => "commit",
        GitObject::Tree { .. } => "tree",
        GitObject::Tag { .. } => "tag",
    }
}


impl GitObject {
    pub fn new(data: Vec<u8>, type_: String) -> GitObject {
        
        match type_.as_str() {
            "blob" => GitObject::Blob { raw_data: data },
            "tree" => GitObject::Tree { raw_data: data },
            "commit" => {
                create_commit(data)
            },
            "tag" => GitObject::Tag { raw_data: data },
            _ => panic!("Unknown type {}", type_)
        }
    }

    pub fn write(&self, repo: &Repository) -> Result<()> {
        let data: Vec<u8> = match self {
            GitObject::Blob { raw_data } => { raw_data.clone() }
            GitObject::Tree { raw_data } => { raw_data.clone() }
            GitObject::Tag { raw_data } => { raw_data.clone() }
            GitObject::Commit { raw_data, .. } => { raw_data.clone() }
        };

        let (before, after) = &data.split_at(2);
        let path = repo.repo_create_file_vec(vec!["objects", &*String::from_utf8(before.to_vec()).unwrap(), &*String::from_utf8(after.to_vec()).unwrap()]).unwrap();
        let result = self.serialize();

        if !path.exists() {
            let mut f = File::create(path).unwrap();
            f.write_all(miniz_oxide::deflate::compress_to_vec_zlib(result.as_bytes(), 1).as_slice()).unwrap();
            Ok(())
        } else {
            return Err(anyhow!("Object already exists"));
        }
    }

    fn hash(&self) -> String {
        let mut hasher = Sha1::new();
        hasher.update(self.serialize().as_bytes());
        let finalized = hasher.finalize_fixed();
        hex::encode(finalized)
    }

    pub fn serialize(&self) -> String {
        let data: Vec<u8> = match self {
            GitObject::Blob { raw_data } => { raw_data.clone() }
            GitObject::Tree { raw_data } => { raw_data.clone() }
            GitObject::Tag { raw_data } => { raw_data.clone() }
            GitObject::Commit { raw_data, .. } => { raw_data.clone() }
        };
        let result = format!("{} {}\0{}", get_git_type(&self), &data.len(), String::from_utf8(data.clone()).unwrap());
        result
    }
}

impl Display for GitObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitObject::Blob { raw_data } => {
                write!(f, "{}", String::from_utf8(raw_data.clone()).unwrap())
            }
            GitObject::Tree { raw_data } => {
                write!(f, "{}", String::from_utf8(raw_data.clone()).unwrap())
            }
            GitObject::Tag { raw_data } => {
                write!(f, "{}", String::from_utf8(raw_data.clone()).unwrap())
            }
            GitObject::Commit { raw_data, .. } => {
                write!(f, "{}", String::from_utf8(raw_data.clone()).unwrap())
            }
        }
    }
}


/// Creates a commit object from a byte array (decompressed and processed).
fn create_commit(data: Vec<u8>) -> GitObject {
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
    GitObject::Commit { raw_data: data, tree, parent, author, committer, gpgsig, message }
}


/// Returns a GitObject from a SHA1 hash provided a repository exists.
pub fn read_from_sha1(repo: &Repository, sha: &String) -> Result<GitObject> {
    let path = repo.repo_git_path_vec(vec!["objects", &sha[0..2], &sha[2..]]);

    if !path.exists() {
        return Err(anyhow!("Object does not exist {}", sha));
    } else if !path.is_file() {
        return Err(anyhow!("Object isn't a file {}", sha));
    }

    let mut f = File::open(path)?;
    let mut buf = vec![];
    f.read_to_end(&mut buf)?;
    parse_from_raw(buf)
}


/// Parses a GitObject from a byte array.
/// `<blob|tree|commit|tag> <len_data>\0<data>`
pub fn parse_from_raw(data: Vec<u8>) -> Result<GitObject> {
    let raw: Vec<u8> = match miniz_oxide::inflate::decompress_to_vec_zlib(&data) {
        Ok(v) => { v }
        Err(_) => { return Err(anyhow!("Failed to decompress data")); }
    };


    let x: usize = match raw.iter().position(|x| *x == 32u8) {
        Some(v) => { v }
        None => { return Err(anyhow!("Failed to find space")); }
    };

    let fmt = match String::from_utf8(raw[0..x].to_vec()) {
        Ok(v) => { v }
        Err(_) => { return Err(anyhow!("Failed to parse format")); }
    };

    let y = 1 + x + raw[x+1..].iter().position(|x| *x == 0u8).context("Failed to find 0 byte")?; //+1 because \0 is 2 wide
    let size = String::from_utf8(raw[x+1..y].to_vec()).context("Failed to parse the size of the data")?.parse::<usize>().unwrap();

    if size != raw.len() -y - 1{
        return Err(anyhow!("Malformed object"));
    }

    parse_from_bytes(&fmt, raw[y+1..].to_vec())
}


/// takes in the processed and validated data and the type of object then returns it as an enum
pub fn parse_from_bytes(fmt: &str, data: Vec<u8>) -> Result<GitObject> {
    match fmt {
        "commit" => {
            Ok(create_commit(data))
        },
        "blob" => Ok(GitObject::Blob { raw_data: data }),
        "tree" => Ok(GitObject::Tree { raw_data: data }),
        "tag" => Ok(GitObject::Tag { raw_data: data }),
        _ => Err(anyhow!("Unknown type {}", fmt)),
    }
}
