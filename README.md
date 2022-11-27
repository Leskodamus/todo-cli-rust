# todo-cli
A lightweight and super fast cli todo program written in Rust

Currently supported operating systems:

* Linux
* macOS
* Windows

Fork of <https://github.com/sioodmy/todo>

## usage

```Usage: todo [COMMAND] [ARGUMENTS]
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
```
