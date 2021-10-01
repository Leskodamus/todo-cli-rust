use colored::*;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::prelude::Read;
use std::io::{BufReader, BufWriter, Write};
use std::{process, env, path};

pub struct Todo {
    pub todo: Vec<String>,
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
            .open(todo_path)
            .expect("couldn't open the todo file");

        let mut buf_reader = BufReader::new(&todo_file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents).unwrap();
        let todo = contents.lines().map(str::to_string).collect();

        Ok(Self { todo })
    }

    pub fn list(&self) -> () {
        for (number, task) in self.todo.iter().enumerate() {
            let number = (number + 1).to_string().bold();

            // saves the symbol of current task
            let symbol = &task[..4];
            // saves a task without a symbol
            let task = &task[4..];

            // checks if the current task is completed or not...
            if symbol == "[*] " {
                // DONE
                // if the task is completed, then it prints it with a strikethrough
                println!("{} {}", number, task.strikethrough());
            } else if symbol == "[ ] " {
                // NOT DONE
                // if the task is not completed yet, then it will print it as it is
                println!("{} {}", number, task);
            }
        }
    }

    // This one is for yall, dmenu chads <3
    //pub fn raw(&self, arg: &[String]) {
    //    if arg.len() > 1 {
    //        eprintln!("todo raw takes only 1 argument, not {}", arg.len())
    //    } else if arg.len() < 1 {
    //        eprintln!("todo raw takes 1 argument (done/todo)");
    //    } else {
    //        // This loop will repeat itself for each taks in TODO file
    //        for task in self.todo.iter() {
    //            if task.len() > 5 {
    //                // Saves the symbol of current task
    //                let symbol = &task[..4];
    //                // Saves a task without a symbol
    //                let task = &task[4..];

    //                // Checks if the current task is completed or not...
    //                if symbol == "[*] " && arg[0] == "done" {
    //                    // DONE
    //                    //If the task is completed, then it prints it with a strikethrough
    //                    println!("{}", task);
    //                } else if symbol == "[ ] " && arg[0] == "todo" {
    //                    // NOT DONE

    //                    //If the task is not completed yet, then it will print it as it is
    //                    println!("{}", task);
    //                }
    //            }
    //        }
    //    }
    //}

    pub fn add(&self, args: &[String]) {
        if args.len() < 1 {
            eprintln!("todo add takes at least 1 argument");
            process::exit(1);
        } else {
            let mut home: OsString;

            if cfg!(windows) {
                home = env::var_os("USERPROFILE").unwrap();
            } else {
                home = env::var_os("XDG_DATA_HOME").unwrap_or(env::var_os("HOME").unwrap());
            }

            let todo_path = path::Path::new(&home).join(".todo");
    
            let todo_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(todo_path)
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
            eprintln!("todo rm takes at least 1 argument");
            process::exit(1);
        } else {
            let mut home: OsString;

            if cfg!(windows) {
                home = env::var_os("USERPROFILE").unwrap();
            } else {
                home = env::var_os("XDG_DATA_HOME").unwrap_or(env::var_os("HOME").unwrap());
            }

            let todo_path = path::Path::new(&home).join(".todo");

            let todo_file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(todo_path)
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

    // pub fn sort(&self) {
    //     // Creates a new empty string
    //     let newtodo: String;

    //     let mut todo = String::new();
    //     let mut done = String::new();

    //     for line in self.todo.iter() {
    //         if line.len() > 5 {
    //             if &line[..4] == "[ ] " {
    //                 let line = format!("{}\n", line);
    //                 todo.push_str(&line);
    //             } else if &line[..4] == "[*] " {
    //                 let line = format!("{}\n", line);
    //                 done.push_str(&line);
    //             }
    //         }
    //     }

    //     newtodo = format!("{}{}", &todo, &done);
    //     // Opens the TODO file with a permission to:
    //     let mut todofile = OpenOptions::new()
    //         .write(true) // a) write
    //         .truncate(true) // b) truncrate
    //         .open("TODO")
    //         .expect("Couldn't open the todo file");

    //     // Writes contents of a newtodo variable into the TODO file
    //     todofile
    //         .write_all(newtodo.as_bytes())
    //         .expect("error while trying to save the todofile");
    // }

    pub fn done(&self, args: &[String]) {
        if args.len() < 1 {
            eprintln!("todo done takes at least 1 argument");
            process::exit(1);
        } else {
            let mut home: OsString;

            if cfg!(windows) {
                home = env::var_os("USERPROFILE").unwrap();
            } else {
                home = env::var_os("XDG_DATA_HOME").unwrap_or(env::var_os("HOME").unwrap());
            }

            let todo_path = path::Path::new(&home).join(".todo");

            let todo_file = OpenOptions::new()
                .write(true)
                .open(todo_path)
                .expect("couldn't open the todo file");

            let mut buffer = BufWriter::new(todo_file);

            for (pos, line) in self.todo.iter().enumerate() {
                if line.len() > 5 {
                    if args.contains(&(pos + 1).to_string()) {
                        if &line[..4] == "[ ] " {
                            let line = format!("[*] {}\n", &line[4..]);
                            buffer
                                .write_all(line.as_bytes())
                                .expect("unable to write data");
                        } else if &line[..4] == "[*] " {
                            let line = format!("[ ] {}\n", &line[4..]);
                            buffer
                                .write_all(line.as_bytes())
                                .expect("unable to write data");
                        }
                    } else {
                        if &line[..4] == "[ ] " || &line[..4] == "[*] " {
                            let line = format!("{}\n", line);
                            buffer
                                .write_all(line.as_bytes())
                                .expect("unable to write data");
                        }
                    }
                }
            }
        }
    }
}

const TODO_HELP: &str = "Usage: todo [COMMAND] [ARGUMENTS]
Todo is a super fast and simple tasks organizer written in rust
Example: todo list
Available commands:
    - add [TASK/s] 
        adds new task/s
        Example: todo add \"buy carrots\"
    - list
        lists all tasks
        Example: todo list
    - done [INDEX]
        marks task as done
        Example: todo done 2 3 (marks second and third tasks as completed)
    - rm [INDEX] 
        removes a task
        Example: todo rm 4 
    - sort
        sorts completed and uncompleted tasks
        Example: todo sort 
    - raw [todo/done]
        prints nothing but done/incompleted tasks in plain text, useful for scripting
        Example: todo raw done
";

pub fn help() {
    println!("{}", TODO_HELP);
}
