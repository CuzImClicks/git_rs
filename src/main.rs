use std::env;
use std::path::PathBuf;

use crate::object::{OBJECT_TYPES, read_git_object};
use crate::repository::find_repo;
use crate::utils::adjust_canonicalization;

mod repository;
mod utils;
mod object;

fn main() {

    let mut args: Vec<String> = env::args().collect();

    args.remove(0);

    if args.is_empty() {
        println!("git_rs");
        return;
    }

    let mut repo = repository::Repository::new(find_repo(env::current_dir().unwrap()).unwrap());

    match &*args[0] {
        "add" => {

        }
        "cat-file" => {
            match args.len() {
                 x if x <= 2 => {
                    eprintln!("Error: Not enough arguments provided!");
                }
                3 => {
                    if !OBJECT_TYPES.contains(&&*args[1]) {
                        eprintln!("Error: Invalid object type provided!");
                    }
                    println!("{}", String::from_utf8(read_git_object(&repo, args[2].to_string()).unwrap().serialize()).unwrap());
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

        }
        "init" => {
            let mut r = repository::Repository::new(PathBuf::from(if args.len() == 1 { "." } else { &*args[1] }));
            match r.create() {
                Ok(_) => println!("Initialized empty Git repository in {}", adjust_canonicalization(&r.gitdir)),
                Err(e) => eprintln!("Error: {}", e)
            }
        }
        "log" => {
            
        }
        "ls-files" => {
            
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
            println!("Invalid argument provided!");
        }
    }
}
