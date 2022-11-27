use std::{env, process};
use todo::{Todo, help};

fn main() {
    let mut todo = match Todo::new() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Could not create todo instance: {}", e);
            process::exit(1);
        },
    };

    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() >= 1 {
        let cmd = &args[0];

        match &cmd[..] {
            "list" => todo.list(),
            "add" => todo.add(&args[1..]),
            "rm" => todo.remove(&args[1..]),
            "done" => todo.done(&args[1..]),
            "undone" => todo.undone(&args[1..]),
            "sort" => todo.sort(),
            "raw" => todo.raw(&args[1..]),
            "help" | "--help" | "-h" | _ => help(),
        }
    } else {
        todo.list();
    }
}
