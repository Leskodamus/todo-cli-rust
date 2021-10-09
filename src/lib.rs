use colored::*;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::prelude::Read;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::{process, env, path};

pub struct Todo {
    pub todo: Vec<String>,
    pub todo_path: PathBuf,
}

impl Todo {
    pub fn new() -> Result<Self, String> {
        let mut home: OsString;

        if cfg!(windows) {
            home = env::var_os("USERPROFILE").unwrap();
        } else {
            home = env::var_os("XDG_DATA_HOME").unwrap_or(env::var_os("HOME").unwrap());
        }

        let todo_path = path::Path::new(&home).join(".todo");
        let todo_file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(&todo_path)
            .expect("couldn't open the todo file");

        let mut buf_reader = BufReader::new(&todo_file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents).unwrap();
        let todo = contents.lines().map(str::to_string).collect();

        Ok(Self { todo, todo_path })
    }

    pub fn list(&self) -> () {
        for (number, task) in self.todo.iter().enumerate() {
            let number = (number + 1).to_string().bold();

            // check length of current task
            if task.len() <= 4 { 
                eprintln!("{} corrupt todo file: task with wrong format\nfix your todo file: {}", "warning:".red(), self.todo_path.display());
                process::exit(1);
            }

            // saves the symbol of current task => '[ ] ' or '[x] '
            let symbol = &task[..4];

            // saves a task without the symbol
            let task = &task[4..];

            match symbol {
                "[*] " => println!("{} {}", number, task.strikethrough()),  /* DONE */
                "[ ] " => println!("{} {}", number, task),  /* NOT DONE */
                _ => eprintln!("{} possibility of broken todo file", "warning:".red()),   /* SMTH WRONG */
            }
        }
    }

    pub fn raw(&self, arg: &[String]) {
        if arg.len() > 1 {
            eprintln!("todo raw takes only 1 argument, not {}", arg.len())
        } else if arg.len() < 1 {
            eprintln!("todo raw needs 1 argument [done|todo]");
        } else {
            for task in self.todo.iter() {
                if task.len() > 4 {
                    // save the symbol of current task
                    let symbol = &task[..4];

                    // save a task without the symbol
                    let task = &task[4..];

                    match symbol {
                        "[ ] " if arg[0] == "undone" => println!("{}", task),     /* DONE */
                        "[*] " if arg[0] == "done" => println!("{}", task),   /* NOT DONE */
                        "[ ] " | "[*] " => (),  /* do nothing; only show for appropriate arg */
                        _ => eprintln!("{} possibility of broken todo file", "warning:".red()),     /* SMTH WRONG */
                    }
                } else {
                    eprintln!("{} corrupt todo file: task with wrong format\nfix your todo file: {}", "warning:".red(), self.todo_path.display());
                    process::exit(1);
                }
            }
        }
    }

    pub fn add(&self, args: &[String]) {
        if args.len() < 1 {
            eprintln!("todo add needs at least 1 argument");
            process::exit(1);
        } else {
            let todo_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.todo_path)
                .expect("couldn't open the todo file");

            let mut buffer = BufWriter::new(todo_file);
            for arg in args {
                if arg.trim().len() < 1 {
                    continue;
                }

                let line = format!("[ ] {}\n", arg);
                buffer.write_all(line.as_bytes()).expect("unable to write data");
            }
        }
    }

    pub fn remove(&self, args: &[String]) {
        if args.len() < 1 {
            eprintln!("todo rm needs at least 1 argument");
            process::exit(1);
        } else {
            let todo_file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&self.todo_path)
                .expect("couldn't open the todo file");

            let mut buffer = BufWriter::new(todo_file);

            for (pos, line) in self.todo.iter().enumerate() {
                if args.contains(&(pos + 1).to_string()) {
                    continue;
                }

                let line = format!("{}\n", line);
                buffer.write_all(line.as_bytes()).expect("unable to write data");
            }
        }
    }

    pub fn done(&self, args: &[String]) {
        if args.len() < 1 {
            eprintln!("todo done needs at least 1 argument");
            process::exit(1);
        } else {
            let todo_file = OpenOptions::new()
                .write(true)
                .open(&self.todo_path)
                .expect("couldn't open the todo file");

            let mut buffer = BufWriter::new(todo_file);

            for (pos, line) in self.todo.iter().enumerate() {
                if line.len() > 4 {
                    if args.contains(&(pos + 1).to_string()) {
                        let line = format!("[*] {}\n", &line[4..]);
                        buffer.write_all(line.as_bytes()).expect("unable to write data");
                    } else {
                        let line = format!("{}\n", line);
                        buffer.write_all(line.as_bytes()).expect("unable to write data");
                    }
                } else {
                    eprintln!("{} corrupt todo file: task with wrong format\nfix your todo file: {}", "warning:".red(), self.todo_path.display());
                    process::exit(1);
                }
            }
        }
    }

    pub fn undone(&self, args: &[String]) {
        if args.len() < 1 {
            eprintln!("todo undone needs at least 1 argument");
            process::exit(1);
        } else {
            let todo_file = OpenOptions::new()
                .write(true)
                .open(&self.todo_path)
                .expect("couldn't open the todo file");

            let mut buffer = BufWriter::new(todo_file);

            for (pos, line) in self.todo.iter().enumerate() {
                if line.len() > 4 {
                    if args.contains(&(pos + 1).to_string()) {
                        if &line[..4] == "[*] " {
                            let line = format!("[ ] {}\n", &line[4..]);
                            buffer.write_all(line.as_bytes()).expect("unable to write data");
                        }
                    } else {
                        let line = format!("{}\n", line);
                        buffer.write_all(line.as_bytes()).expect("unable to write data");
                    }
                } else {
                    eprintln!("{} corrupt todo file: task with wrong format\nfix your todo file: {}", "warning:".red(), self.todo_path.display());
                    process::exit(1);
                }
            }
        }
    }

    pub fn sort(&self) {
        let mut new_todo = Vec::<String>::new();

        for line in self.todo.iter() {
            if line.len() > 4 {
                match &line[..4] {
                    "[ ] " => new_todo.insert(0, format!("{}\n", line)),    /* put done task to beginning of vec */
                    "[*] " => new_todo.push(format!("{}\n", line)),     /* add undone task to vec */
                    _ => eprintln!("{} possibility of broken todo file", "warning:".red()),     /* SMTH WRONG */
                }
            } else {
                eprintln!("{} corrupt todo file: task with wrong format\nfix your todo file: {}", "warning:".red(), self.todo_path.display());
                process::exit(1);
            }
        }

        let todo_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.todo_path)
            .expect("couldn't open the todo file");

        let mut buffer = BufWriter::new(todo_file);
        for line in new_todo {
            buffer.write_all(line.as_bytes()).expect("error while trying to write to todo file");
        }
    }
}

const TODO_HELP: &str = "Usage: todo [COMMAND] [ARGUMENTS]
todo is a super fast and simple tasks organizer written in rust

Available commands:
    - add [TASK/s] 
        adds new task/s
        Example: todo add \"read a book\"
    - list
        lists all tasks
        Example: todo list
    - done [INDEX]
        marks task as done
        Example: todo done 2 3 (marks second and third tasks as completed)
    - undone [INDEX]
        reverts done task to undone
        Example: todo undone 3 (no longer marks the third task as completed)
    - rm [INDEX] 
        removes a task
        Example: todo rm 4 
    - sort
        sorts completed and uncompleted tasks
        Example: todo sort 
    - raw [todo|done]
        prints nothing but done/undone tasks in plain text, useful for scripting
        Example: todo raw done
";

pub fn help() {
    println!("{}", TODO_HELP);
}
