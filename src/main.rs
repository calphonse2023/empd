#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use owo_colors::OwoColorize;

use std::{
    fs::{self},
    io::{self, ErrorKind},
    path::{self, Path},
};

use clap::Parser;

/// Checks if a directory or file is empty, or if a symbolic link points to a path that does not exist. Only supports UTF-8 paths.
#[derive(Parser)]
#[command(author, version, about)]
struct EmpdArgs {
    /// Delete the file or directory if it is empty
    #[arg(short, long)]
    delete_if_empty: bool,
    /// Path to test
    #[arg(index = 1)]
    path: String,
}

const CHECK_MARK: &str = "✔️";
const X: &str = "🗙";

#[allow(clippy::too_many_lines)]
fn main() {
    let EmpdArgs {
        delete_if_empty,
        path,
    } = EmpdArgs::parse();

    let path_path = path::Path::new(&path);

    let path_path_str = path_path
        .to_str()
        .expect("Could not convert path to a UTF-8 string");

    let result = fs::symlink_metadata(path_path);

    let exit_code = match result {
        Err(er) => match er.kind() {
            ErrorKind::NotFound => {
                eprintln!("Path \"{}\" does not exist", path_path_str.bold());

                11
            }
            ErrorKind::PermissionDenied => {
                eprintln!("Permission to path \"{}\" was denied", path_path_str.bold());

                12
            }
            _ => {
                panic!("{er}");
            }
        },
        Ok(me) => {
            match me {
                me if me.is_dir() => {
                    let canonicalize_result = canonicalize(path_path_str, path_path)
                        .expect("Could not canonicalize directory path");

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
                                panic!(
                                    "Encountered directory entry that is not a directory, file, or symlink"
                                );
                            }
                        }
                    }

                    let total_items = directories + files + symlinks;

                    if total_items > 0 {
                        println!(
                            " {}  Path \"{}\" is a {} (directories: {}, files: {}, symlinks: {}, total items: {})",
                            X.bold().red(),
                            canonicalize_result.bold(),
                            "non-empty directory".bold().red(),
                            bold_if_greater_than_zero(directories),
                            bold_if_greater_than_zero(files),
                            bold_if_greater_than_zero(symlinks),
                            bold_if_greater_than_zero(total_items)
                        );

                        31
                    } else {
                        println!(
                            " {}  Path \"{}\" is an {}",
                            CHECK_MARK.bold().green(),
                            canonicalize_result.bold(),
                            "empty directory".bold().green()
                        );

                        if delete_if_empty {
                            eprintln!(
                                "Are you sure you want to delete empty directory \"{}\"? (\"y\")\n\
                                (Note that no file locking or revalidation is performed, and the directory may be non-empty by the time you respond to this prompt!)",
                                canonicalize_result.bold()
                            );

                            let input = &mut String::new();

                            io::stdin()
                                .read_line(input)
                                .expect("\"read_line\" call failed");

                            if input == "y\n" {
                                // TODO Status of path could have changed by now
                                fs::remove_dir(path_path).unwrap();

                                println!(
                                    "Deleted empty directory \"{}\"",
                                    canonicalize_result.bold()
                                );

                                0
                            } else {
                                println!("Input was not \"y\", not deleting empty directory");

                                32
                            }
                        } else {
                            0
                        }
                    }
                }
                me if me.is_file() => {
                    let canonicalize_result = canonicalize(path_path_str, path_path)
                        .expect("Could not canonicalize file path");

                    let len = me.len();

                    if len > 0 {
                        println!(
                            " {}  Path \"{}\" is a {} (bytes: {})",
                            X.bold().red(),
                            canonicalize_result.bold(),
                            "non-empty file".bold().red(),
                            len.bold()
                        );

                        21
                    } else {
                        println!(
                            " {}  Path \"{}\" is an {}",
                            CHECK_MARK.bold().green(),
                            canonicalize_result.bold(),
                            "empty file".bold().green()
                        );

                        if delete_if_empty {
                            eprintln!(
                                "Are you sure you want to delete empty file \"{}\"? (\"y\")\n\
                                (Note that no file locking or revalidation is performed, and the file may be non-empty by the time you respond to this prompt!)",
                                canonicalize_result.bold()
                            );

                            let input = &mut String::new();

                            io::stdin()
                                .read_line(input)
                                .expect("\"read_line\" call failed");

                            if input == "y\n" {
                                // TODO Status of path could have changed by now
                                fs::remove_file(path_path).unwrap();

                                println!("Deleted empty file \"{}\"", canonicalize_result.bold());

                                0
                            } else {
                                println!("Input was not \"y\", not deleting empty file");

                                22
                            }
                        } else {
                            0
                        }
                    }
                }
                me if me.is_symlink() => {
                    let link_path_buf =
                        path_path.read_link().expect("Could not read symbolic link");

                    let link_path_buf_str = link_path_buf
                        .to_str()
                        .expect("Could not convert symbolic link path to a UTF-8 string");

                    let canonicalize_result = canonicalize(path_path_str, path_path);

                    #[allow(clippy::single_match_else)]
                    {
                        match canonicalize_result {
                            Some(st) => {
                                println!(
                                    " {}  Path \"{}\" (non-canonicalized) is a symbolic link to \"{}\" (resolves to \"{st}\")",
                                    X.bold().red(),
                                    path_path_str.bold(),
                                    link_path_buf_str.bold()
                                );

                                41
                            }
                            None => {
                                println!(
                                    " {}  Path \"{}\" (non-canonicalized) is a symbolic link to non-existent file \"{}\" (non-canonicalized)",
                                    CHECK_MARK.bold().green(),
                                    path_path_str.bold(),
                                    link_path_buf_str.bold()
                                );

                                if delete_if_empty {
                                    eprintln!(
                                        "Are you sure you want to delete symbolic link \"{}\" (non-canonicalized) pointing to non-existent file \"{}\"? (non-canonicalized) (\"y\")\n\
                                        (Note that no file locking or revalidation is performed, and the symbolic link destination may exist by the time you respond to this prompt!)",
                                        path_path_str.bold(),
                                        link_path_buf_str.bold()
                                    );

                                    let input = &mut String::new();

                                    io::stdin()
                                        .read_line(input)
                                        .expect("\"read_line\" call failed");

                                    if input == "y\n" {
                                        // TODO Status of path could have changed by now
                                        fs::remove_file(path_path).unwrap();

                                        println!(
                                            "Deleted symbolic link \"{}\" (non-canonicalized)",
                                            path_path_str.bold()
                                        );

                                        0
                                    } else {
                                        println!("Input was not \"y\", not deleting symbolic link");

                                        42
                                    }
                                } else {
                                    0
                                }
                            }
                        }
                    }
                }
                _ => {
                    panic!("Path \"{path_path_str}\" is not a directory, file, or symlink")
                }
            }
        }
    };

    if exit_code != 0 {
        eprintln!("Exiting with non-zero exit code {}", exit_code.bold());
    }

    std::process::exit(exit_code);
}

fn bold_if_greater_than_zero(input: i32) -> String {
    if input > 0 {
        input.bold().to_string()
    } else {
        input.to_string()
    }
}

fn canonicalize(path_str: &str, path_path: &Path) -> Option<String> {
    let canonicalize_result = fs::canonicalize(path_path);

    let option: Option<String> = match canonicalize_result {
        Ok(pa) => {
            let path_buf_str = pa
                .to_str()
                .expect("Could not convert path to a UTF-8 string");

            eprintln!(
                "Canonicalized input path \"{}\" to \"{}\"",
                path_str.bold(),
                path_buf_str.bold()
            );

            Some(path_buf_str.to_owned())
        }
        Err(er) => match er.kind() {
            ErrorKind::NotFound => {
                eprintln!(
                        "Could not canonicalize input path \"{}\" because it or the file it resolves to does not exist",
                        path_str.bold()
                    );

                None
            }
            _ => {
                panic!("{er}");
            }
        },
    };

    option
}
