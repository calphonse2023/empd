use owo_colors::OwoColorize;

use std::{
    fs::{self},
    io, path,
};

use clap::Parser;

/// Checks if a directory (or file) is empty. Only supports UTF-8 paths.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct EmpdArgs {
    /// Path to test
    #[arg(index = 1)]
    path: String,
}

const CHECK_MARK: &str = "✔️";
const X: &str = "🗙";

fn main() {
    let EmpdArgs { path } = EmpdArgs::parse();

    let path_path = path::Path::new(&path);

    let result = fs::symlink_metadata(&path_path);

    match result {
        Err(er) => match &er.kind() {
            io::ErrorKind::NotFound => {
                println!("Path \"{}\" does not exist", path.bold());

                std::process::exit(1);
            }
            io::ErrorKind::PermissionDenied => {
                println!("Permission to path \"{}\" was denied", path.bold());

                std::process::exit(2);
            }
            _ => {
                panic!("{}", er);
            }
        },
        Ok(me) => {
            let path_buf = fs::canonicalize(path_path).expect("Could not canonicalize path");

            let path_buf_str = (&path_buf)
                .to_str()
                .expect("Could not convert path to a UTF-8 string");

            println!(
                "Canonicalized input path \"{}\" to \"{}\"",
                path.bold(),
                path_buf_str.bold()
            );

            match me {
                me if me.is_dir() => {
                    let read_dir = path_path.read_dir().expect("Could not read directory");

                    let mut directories = 0;
                    let mut files = 0;
                    let mut symlinks = 0;

                    for re in read_dir {
                        let di = re.expect("Could not access directory entry");

                        let fi = di
                            .file_type()
                            .expect("Could not get the directory entry's file type");

                        match fi {
                            fi if fi.is_dir() => {
                                directories += 1;
                            }
                            fi if fi.is_file() => {
                                files += 1;
                            }
                            fi if fi.is_symlink() => {
                                symlinks += 1;
                            }
                            _ => {
                                panic!("Encountered directory entry that is not a directory, file, or symlink")
                            }
                        }
                    }

                    let total_items = directories + files + symlinks;

                    if total_items > 0 {
                        println!(
                            " {}  Path \"{}\" is a {} (directories: {}, files: {}, symlinks: {}, total items: {})",
                            X.bold().red(),
                            path_buf_str.bold(),
                            "non-empty directory".bold().red(),
                            bold_if_greater_than_zero(directories),
                            bold_if_greater_than_zero(files),
                            bold_if_greater_than_zero(symlinks),
                            bold_if_greater_than_zero(total_items)
                        );

                        std::process::exit(3);
                    } else {
                        println!(
                            " {}  Path \"{}\" is an {}",
                            CHECK_MARK.bold().green(),
                            path_buf_str.bold(),
                            "empty directory".bold().green()
                        );

                        std::process::exit(0);
                    }
                }
                me if me.is_file() => {
                    let len = me.len();

                    if len > 0 {
                        println!(
                            " {}  Path \"{}\" is a {} (bytes: {})",
                            X.bold().red(),
                            path_buf_str.bold(),
                            "non-empty file".bold().red(),
                            len.bold()
                        );

                        std::process::exit(4);
                    } else {
                        println!(
                            " {}  Path \"{}\" is an {}",
                            CHECK_MARK.bold().green(),
                            path_buf_str.bold(),
                            "empty file".bold().green()
                        );

                        std::process::exit(0);
                    }
                }
                me if me.is_symlink() => {
                    let link_path_buf =
                        path_path.read_link().expect("Could not read symbolic link");

                    let link_path_buf_str = link_path_buf
                        .to_str()
                        .expect("Could not convert symbolic link path to a UTF-8 string");

                    println!(
                        " {}  Path \"{}\" is a symbolic link to \"{}\"",
                        X.bold().red(),
                        path_buf_str.bold(),
                        link_path_buf_str
                    );

                    std::process::exit(5);
                }
                _ => {
                    panic!(
                        "Path \"{}\" is not a directory, file, or symlink",
                        path_buf_str
                    )
                }
            }
        }
    }
}

fn bold_if_greater_than_zero(input: i32) -> String {
    if input > 0 {
        input.bold().to_string()
    } else {
        input.to_string()
    }
}
