use std::fs::File;
use std::io::Read;
use anyhow::{anyhow, Context, Result};
use crate::enums::{create_commit, create_tree, GitObject};
use crate::repository::Repository;

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
/// ```
/// <blob|tree|commit|tag> <len_data>\0<data>
/// ```
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


/// Takes in the processed and validated data and the type of object then returns it as an enum
pub fn parse_from_bytes(fmt: &str, data: Vec<u8>) -> Result<GitObject> {
    match fmt {
        "commit" => {
            Ok(create_commit(data))
        },
        "blob" => Ok(GitObject::Blob { raw_data: data }),
        "tree" => {
            Ok(create_tree(data))
        },
        "tag" => Ok(GitObject::Tag { raw_data: data }),
        _ => Err(anyhow!("Unknown type {}", fmt)),
    }
}
