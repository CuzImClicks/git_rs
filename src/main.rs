use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use anyhow::{anyhow, Context, Result};
use crate::enums::{GitObject};
use crate::functions::read_from_sha1;

use crate::repository::find_repo;
use crate::utils::{adjust_canonicalization, Set};

mod repository;
mod utils;
mod enums;
mod functions;

fn main() -> Result<()> {

    let mut args: Vec<String> = env::args().collect();

    args.remove(0);

    if args.is_empty() {
        println!("git_rs");
        return Ok(());
    }

    let mut repo = repository::Repository::new(find_repo(env::current_dir().unwrap()).unwrap());

    match &*args[0] {
        "add" => {

        }
        "cat-file" => {
            // cat-file <object hash>
            match args.len() {
                 x if x <= 1 => {
                     return Err(anyhow!("Not enough arguments provided!\nUsage: git cat-file <object>"));
                }
                2 => {
                    println!("{}", read_from_sha1(&repo, &args[1].to_string()).context("Failed to read object from sha1")?);
                }
                _ => {

                }
            }
            
        }
        "check-ignore" => {
            
        }
        "checkout" => {
            
        }
        "commit" => {
            
        }
        "hash-object" => {
            // hash-object [-w] [-t TYPE] FILE
            match args.len() {
                x if x <= 1 => {
                    return Err(anyhow!("Not enough arguments provided!\nUsage: hash-object [-w] [-t TYPE] FILE"));
                }
                x if (2..=4).contains(&x) => {
                    let t: String = if let Some(i) = args.iter().position(|x| x == "-t") {
                        args.get(i + 1).unwrap_or(&"blob".to_string()).clone()
                    } else {
                        "blob".to_string()
                    };
                    let write: bool = args.contains(&"-w".to_string());
                    let path = repo.repo_path(&args[args.len() - 1]);
                    if !path.exists() {
                        return Err(anyhow!("File '{}' does not exist!", path.display()));
                    }

                    let mut file = File::open(path).unwrap();
                    let mut buf = vec![];
                    file.read_to_end(&mut buf).unwrap();
                    let obj: GitObject = GitObject::new(buf, t);
                    if write {
                        if let Err(e) = obj.write(&repo) {
                            return Err(anyhow!("Error writing object: {}", e));
                        }
                    } else {
                        println!("{}", obj.serialize());
                    }
                }
                _ => {}
            }
        }
        "init" => {
            let mut r = repository::Repository::new(PathBuf::from(if args.len() == 1 { "." } else { &*args[1] }));
            match r.create() {
                Ok(_) => println!("Initialized empty Git repository in {}", adjust_canonicalization(&r.gitdir)),
                Err(e) => return Err(anyhow!("{}", e))
            }
        }
        "log" => {
            match args.len() {
                x if x != 2 => {
                    return Err(anyhow!("Invalid amount of arguments provided!\nUsage: git log <HEAD>"));
                }
                _ => {}
            }
            let mut parents: Set<String> = Set::new();
            parents.add(args[1].clone());
            while !parents.is_empty() {
                let last = parents.remove(parents.len() - 1);
                let commit_obj = read_from_sha1(&repo, &last).unwrap();
                if let GitObject::Commit { parent, message, .. } = commit_obj {
                    parents.append(&parent.clone());
                    println!("{} - {}", last, message.replace("\n\n", "\n"));
                } else {
                    return Err(anyhow!("Object is not a commit!"));
                }
            }
        }
        "ls-files" => {
            
        }
        "decompress" => {
            let mut file = File::open(&args[1]).unwrap();
            let mut buf: Vec<u8> = vec![];
            file.read_to_end(&mut buf).unwrap();
            let raw = miniz_oxide::inflate::decompress_to_vec_zlib(&buf).unwrap();
            println!("{:?}", raw);
        }
        "ls-tree" => {
            
        }
        "rev-parse" => {
            
        }
        "rm" => {
            
        }
        "show-ref" => {
            
        }
        "status" => {
            
        }
        "tag" => {
            
        }
        _ => {
            return Err(anyhow!("'{:?}' Invalid argument provided!", args));
        }
    }
    Ok(())
}
