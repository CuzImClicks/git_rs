use std::env;
use std::path::PathBuf;
use crate::utils::adjust_canonicalization;

mod repository;
mod utils;

fn main() {

    let mut args: Vec<String> = env::args().collect();

    args.remove(0);

    if args.is_empty() {
        println!("git_rs");
        return;
    }

    match &*args[0] {
        "add" => {

        }
        "cat-file" => {
            
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
            let mut repo = repository::Repository::new(PathBuf::from(if args.len() == 1 { "." } else { &*args[1] }));
            match repo.create() {
                Ok(_) => println!("Initialized empty Git repository in {}", adjust_canonicalization(&repo.gitdir)),
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
