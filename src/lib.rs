use colored::*;
use std::env::temp_dir;
use std::fs::{OpenOptions, remove_file};
use std::io::{BufReader, Write, BufRead, Seek};
use std::path::PathBuf;
use std::process::Command;
use std::{process, env, path::Path};

type Tasks = Vec<String>;

pub struct Todo {
    pub tasks: Tasks,       /* vector containing all tasks */
    pub n_tasks: usize,     /* initial number of tasks */
    pub file_path: PathBuf, /* path of todo file  */
}

impl Todo {
    /* Create a new todo instance which creates the todo file 
     * TODO: use config file and/or env var for file path */
    pub fn new() -> Result<Self, String> {
        let home = match env::consts::OS {
            "linux" | "macos" => {
                env::var_os("XDG_DATA_HOME").unwrap_or(
                    match env::var_os("HOME") {
                        Some(dir) => dir,
                        None => return Err("failed to read $XDG_DATA_HOME or $HOME".into()),
                    })
            },
            "windows" => {
                match env::var_os("USERPROFILE") {
                    Some(dir) => dir,
                    None => return Err("failed to read %USERPROFILE%".into()),
                }
            },
            _ => return Err("unsupported operating system".into()),
        };

        let file_path = Path::new(&home).join(".todo");
        let todo_file = match 
            OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .open(&file_path)
        {
            Ok(f) => f,
            Err(e) => return Err(format!("failed to open todo file: {}", e)),
        };

        let mut tasks = Tasks::new();
        for line in BufReader::new(todo_file).lines() {
            match line {
                Ok(t) => tasks.push(t),
                Err(e) => return Err(format!("failed to read todo file: {}", e)),
            }
        }

        let n_tasks = tasks.len();

        Ok(Self { tasks, n_tasks, file_path })
    }

    /* Check if task (line) format is valid */
    fn is_valid_task(task: &str) -> bool {
        return task.len() > 4 && (&task[..4] == "[ ] " || &task[..4] == "[*] ") 
    }

    /* List all tasks */
    pub fn list(&self) -> () {
        for (idx, task) in self.tasks.iter().enumerate() {
            let idx = (idx + 1).to_string().bold();

            if !Self::is_valid_task(task) {
                eprintln!("corrupt todo file: invalid format at line {}", idx.normal());
                process::exit(2);
            }

            let symbol = &task[..4];
            let task = &task[4..];

            match symbol {
                "[*] " => println!("{} {}", idx, task.strikethrough()), /* DONE */
                "[ ] " => println!("{} {}", idx, task),                 /* NOT DONE */
                _ => {
                    eprintln!("corrupt todo file: invalid format at line {}", idx.normal());
                    process::exit(2);
                }
            }
        }
    }

    /* List either done or undone tasks without any formatting */
    pub fn raw(&self, arg: &[String]) {
        if arg.len() != 1 {
            eprintln!("todo raw needs 1 argument [done|undone]");
            help();
        } else {
            for (idx, task) in self.tasks.iter().enumerate() {
                if Self::is_valid_task(task) {
                    let symbol = &task[..4];
                    let task = &task[4..];
    
                    match symbol {
                        "[ ] " if arg[0] == "undone" => println!("{}", task),   /* DONE */
                        "[*] " if arg[0] == "done" => println!("{}", task),     /* NOT DONE */
                        "[ ] " | "[*] " => (),  /* do nothing; only show for appropriate arg */
                        _ => {
                            eprintln!("corrupt todo file: invalid format at line {}", idx.to_string());
                            process::exit(2);
                        }                    }
                } else {
                    eprintln!("corrupt todo file: invalid format at line {}", idx.to_string());
                    process::exit(2);
                }
            }
        }
    }

    /* Write content of tasks vector to todo file 
     * If append is true, append the file if the vector size has increased */
    fn write_to_file(&self, append: bool) -> Result<(), String> {
        let mut begin = 0;
        let mut file_opts = OpenOptions::new(); 

        if append {
            if self.tasks.len() <= self.n_tasks {
                return Err("cannot append to file because there is nothing to append".into());
            }
            begin = self.n_tasks;
            file_opts.create(true).append(true);
        } else {
            file_opts.create(true).write(true).truncate(true);
        }

        let mut todo_file = match file_opts.open(&self.file_path) {
            Ok(f) => f,
            Err(e) => return Err(format!("failed to open todo file: {}", e)),
        };

        for task in &self.tasks[begin..] {
            match todo_file.write_fmt(format_args!("{task}\n")) {
                Ok(_) => (),
                Err(e) => return Err(format!("failed to write todo file: {}", e)),
            };
        }

        Ok(())
    }

    /* Add new task to todo and save to file */
    pub fn add(&mut self, args: &[String]) {
        if args.len() < 1 {
            eprintln!("todo add needs at least 1 argument");
            process::exit(1);
        } else {
            for task in args {
                self.tasks.push(format!("[ ] {}", task))
            }

            match self.write_to_file(true) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{e}");
                    process::exit(1);
                }
            }
        }
    }

    /* Remove a task from todo */
    pub fn remove(&mut self, args: &[String]) {
        if args.len() < 1 {
            eprintln!("todo rm needs at least 1 argument");
            process::exit(1);
        } else {
            let mut indices = Vec::<usize>::new();
            for arg in args {
                let idx = match arg.parse::<usize>() {
                    Ok(i) => i-1,
                    Err(_) => {
                        eprintln!("todo edit requires a positive integer as argument");
                        process::exit(1);
                    },
                };
                if idx < self.tasks.len() {
                    indices.push(idx);
                }
            }

            /* Sort indices descending to avoid conflicts
             * with indexing when removing tasks */
            indices.sort_by(|a, b| b.cmp(a));
            indices.iter().for_each(|idx| { self.tasks.remove(*idx); () });

            match self.write_to_file(false) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{e}");
                    process::exit(1);
                }
            }
        }
    }

    /* Opens a text editor to edit task */
    fn edit_task_in_editor(&mut self, idx: usize) {
        let file_path = temp_dir().join("EDIT_TODO_TASK");
        let mut tmp_file = match OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .truncate(true)
            .open(&file_path)
        {
            Ok(f) => f,
            Err(_) => {
                eprintln!("failed to open temp file for editing task");
                process::exit(1);
            },
        };

        match tmp_file.write_fmt(format_args!("{}", &self.tasks[idx][4..])) {
            Ok(()) => (),
            Err(e) => {
                eprintln!("failed to initialise temp file for editing task: {e}");
                process::exit(1);
            },
        };

        /* Open tmp_file in editor 
         * TODO: better lookup which editor to use */
        let editor = env::var("EDITOR").unwrap_or("vi".into());
        let editor_status = match Command::new(editor)
            .arg(&file_path)
            .status() 
        {
            Ok(s) => s,
            Err(e) => {
                eprintln!("failed to open editor to edit task: {e}");
                process::exit(1);
            },
        };

        if editor_status.success() {
            /* Reset file pointer to beginning */
            match tmp_file.seek(std::io::SeekFrom::Start(0)) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("failed to read edited task: {e}");
                    process::exit(1);
                },
            };

            let mut new_task = String::new();
            for line in BufReader::new(tmp_file).lines().take(1) {
                match line {
                    Ok(t) => new_task = t,
                    Err(e) => {
                        eprintln!("failed to read edited task: {e}");
                        process::exit(1);
                    },
                }
            }
            self.tasks[idx] = format!("{}{}", &self.tasks[idx][..4], new_task);
            
            match remove_file(file_path) {
                Ok(()) => (),
                Err(e) => {
                    eprintln!("failed to delete temp file: {e}");
                    process::exit(1);
                },
            };
        } else {
            eprintln!("editor failed to process data");
            process::exit(1);
        }
    }

    /* Edit tasks in an editor */
    pub fn edit(&mut self, args: &[String]) {
        if args.is_empty() {
            eprintln!("todo edit needs at least 1 argument");
            process::exit(1);
        } else {
            let mut indices = Vec::<usize>::new();
            for arg in args {
                let idx = match arg.parse::<usize>() {
                    Ok(i) => i-1,
                    Err(_) => {
                        eprintln!("todo edit requires a positive integer as argument");
                        process::exit(1);
                    },
                };
                if idx < self.tasks.len() {
                    indices.push(idx);
                }
            }
            indices.iter().for_each(|idx| self.edit_task_in_editor(*idx));

            /* Sort indices descending to avoid conflicts
             * with indexing when removing empty tasks */
            indices.sort_by(|a, b| b.cmp(a));
            
            /* Delete removed tasks (empty lines) */
            for idx in indices {
                if self.tasks[idx][4..].trim().is_empty() {
                    self.tasks.remove(idx);
                }
            }

            match self.write_to_file(false) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{e}");
                    process::exit(1);
                }
            }
        }
    }

    /* Mark a task as done */
    pub fn done(&mut self, args: &[String]) {
        if args.len() < 1 {
            eprintln!("todo done needs at least 1 argument");
            process::exit(1);
        } else {
            for arg in args {
                let idx = match arg.parse::<usize>() {
                    Ok(i) => i-1,
                    Err(_) => {
                        eprintln!("todo done requires a positive integer as argument");
                        process::exit(1);
                    },
                };
                if idx < self.tasks.len() {
                    let task = &self.tasks[idx];
                    if Self::is_valid_task(task) {
                        self.tasks[idx] = format!("[*] {}", &task[4..]);
                    } else {
                        eprintln!("corrupt todo file: invalid format at line {}", idx.to_string());
                        process::exit(2);
                    }
                }
            }

            match self.write_to_file(false) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{e}");
                    process::exit(1);
                }
            }
        }
    }

    /* Mark a done task as undone */
    pub fn undone(&mut self, args: &[String]) {
        if args.len() < 1 {
            eprintln!("todo undone needs at least 1 argument");
            process::exit(1);
        } else {
            for arg in args {
                let idx = match arg.parse::<usize>() {
                    Ok(i) => i-1,
                    Err(_) => {
                        eprintln!("todo done requires a positive integer as argument");
                        process::exit(1);
                    },
                };
                if idx < self.tasks.len() {
                    let task = &self.tasks[idx];
                    if Self::is_valid_task(task) {
                        self.tasks[idx] = format!("[ ] {}", &task[4..]);
                    } else {
                        eprintln!("corrupt todo file: invalid format at line {}", idx.to_string());
                        process::exit(2);
                    }
                }
            }

            match self.write_to_file(false) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{e}");
                    process::exit(1);
                }
            }
        }
    }

    /* Sort tasks by their status: 
     * done tasks get placed at the bottom */
    pub fn sort(&mut self) {
        for idx in 0..self.tasks.len() {
            let task = &self.tasks[idx];
            if Self::is_valid_task(task) {
                match &task[..4] {
                    "[ ] " => {
                        let task = self.tasks.remove(idx);
                        self.tasks.insert(0, task);
                    },
                    "[*] " => (),
                    _ => {
                        eprintln!("corrupt todo file: invalid format at line {}", idx.to_string());
                        process::exit(2);
                    }
                }
            } else {
                eprintln!("corrupt todo file: invalid format at line {}", idx.to_string());
                process::exit(2);
            }
        }

        match self.write_to_file(false) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{e}");
                process::exit(1);
            }
        }
    }
}

const TODO_HELP: &str = "Usage: todo [COMMAND] [ARGUMENTS]
todo is a super fast and simple tasks organizer written in rust

Available commands:
    - add [TASK]
        adds new task(s)
        Example: todo add \"read a book\" \"do homework\"
    - rm [INDEX] 
        removes task(s) with INDEX 
        Example: todo rm 4 1 (removes first and fourth task)
    - edit [INDEX] (EXPERIMENTAL)
        opens task(s) with INDEX in an editor 
        Example: todo edit 1 2 (opens each task 1 and 2 in an editor)
    - list
        lists all tasks
    - done [INDEX]
        marks task(s) with INDEX as done
        Example: todo done 2 3 (marks second and third task as completed)
    - undone [INDEX]
        reverts done task(s) with INDEX to undone
        Example: todo undone 3 (no longer marks the third task as completed)
    - sort
        sorts completed and uncompleted tasks
    - raw [done|undone]
        prints nothing but done/undone tasks in plain text, useful for scripting
        Example: todo raw done
";

pub fn help() {
    println!("{}", TODO_HELP);
}
