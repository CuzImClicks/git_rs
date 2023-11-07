use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;

use anyhow::{anyhow, Result};
use sha1::{Digest, Sha1};
use sha1::digest::FixedOutput;

use crate::repository::Repository;

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
        raw_data: Vec<u8>,
        leaves: Vec<TreeLeaf>
    },
    Tag {
        raw_data: Vec<u8>
    }
}

#[derive(Debug)]
pub struct TreeLeaf {
    pub mode: String,
    pub name: String,
    pub hash: Vec<u8>
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
            "tree" => {
                create_tree(data)
            },
            "commit" => {
                create_commit(data)
            },
            "tag" => GitObject::Tag { raw_data: data },
            _ => panic!("Unknown type {}", type_)
        }
    }

    /// Writes the serialized and compressed object to the repository.
    /// See: [`GitObject::serialize`]
    pub fn write(&self, repo: &Repository) -> Result<()> {
        let hash = self.hash();
        let (before, after) = hash.split_at(2);
        let path = repo.repo_create_file_vec(vec!["objects", before, after]).unwrap();
        let result = self.serialize();

        if !path.exists() {
            let mut f = File::create(path).unwrap();
            f.write_all(miniz_oxide::deflate::compress_to_vec_zlib(result.as_bytes(), 1).as_slice()).unwrap();
            Ok(())
        } else {
            Err(anyhow!("Object already exists"))
        }
    }

    fn hash(&self) -> String {
        let mut hasher = Sha1::new();
        hasher.update(self.serialize().as_bytes());
        let finalized = hasher.finalize_fixed();
        hex::encode(finalized)
    }

    /// Serializes the object into the git format.
    /// ```
    /// <blob|tree|commit|tag> <len_data>\0<data>
    /// ```
    pub fn serialize(&self) -> String {
        let data: Vec<u8> = match self {
            GitObject::Blob { raw_data } => { raw_data.clone() }
            GitObject::Tree { raw_data, ..} => { raw_data.clone() }
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
            GitObject::Tree { leaves, .. } => {
                write!(f, "{:?}", leaves)
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
pub(crate) fn create_commit(data: Vec<u8>) -> GitObject {
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


pub(crate) fn create_tree(data: Vec<u8>) -> GitObject {
    let mut cursor = 0;
    let mut result: Vec<TreeLeaf> = Vec::new();
    while cursor < data.len() {
        let mut mode = String::new();
        while data[cursor] != 32 {
            mode.push(data[cursor] as char);
            cursor += 1;
        }

        cursor += 1; // skip space

        let mut name = String::new();

        while data[cursor] != 0 {
            name.push(data[cursor] as char);
            cursor += 1;
        }

        cursor += 1; // skip \0

        let mut hash: Vec<u8> = Vec::new();
        for _ in 0..20 {
            hash.push(data[cursor]);
            cursor += 1;
        }

        result.push(TreeLeaf { mode, name, hash });
    }

    GitObject::Tree { raw_data: data, leaves: result }
}
