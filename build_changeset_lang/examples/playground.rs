use build_changeset_lang::*;
use build_script_lang::DefaultSchema;
use build_script_shared::parsers::ParserDeserialize;
use build_script_shared::{BUILDScriptError, InputMarker};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::env;
use std::fs::{create_dir, read_to_string};
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;
use thiserror::Error;

fn version_folder_listener() -> impl FnMut(notify::Result<Event>) -> () {
    let skip = Arc::new(RwLock::new(false));
    move |res| {
        let skip = skip.clone();
        match res {
            Ok(event) => match event.kind {
                EventKind::Modify(_) => {
                    {
                        let mut lock = skip.write().unwrap();
                        let val = *lock;
                        *lock = !val;
                        if val {
                            return;
                        }
                    }
                    let path = event.paths.get(0).unwrap();
                    let filename: usize = path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .split(".")
                        .next()
                        .unwrap()
                        .parse()
                        .unwrap();
                    println!(" ------ Saved {}.bs ------", filename);
                    sleep(Duration::from_millis(1));

                    let content = read_to_string(&path).unwrap();
                    let input = InputMarker::new_from_file(
                        content.as_str(),
                        path.to_str().unwrap().to_string(),
                    );
                    let new_version = DefaultSchema::deserialize(input);
                    match new_version {
                        Ok(new_version) => {
                            let root_path_s = env::var("CARGO_MANIFEST_DIR").unwrap();
                            let root_path = Path::new(&root_path_s);
                            let old_path =
                                root_path.join(format!("examples/versions/{}.bs", filename - 1));
                            if Path::new(&old_path).exists() {
                                let content = read_to_string(&old_path).unwrap();
                                let input = InputMarker::new_from_file(
                                    content.as_str(),
                                    old_path.to_str().unwrap().to_string(),
                                );
                                let old_version = DefaultSchema::deserialize(input);

                                match old_version {
                                    Ok(old_version) => {
                                        let changes = old_version.build_changeset(&new_version);

                                        match changes {
                                            Ok(changes) => {
                                                println!(
                                                    "Changeset for {}.bs -> {}.bs",
                                                    filename - 1,
                                                    filename
                                                );
                                                println!("{}", changes)
                                            }
                                            Err(e) => {
                                                println!("Making changeset failed with:");
                                                println!("{}", e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        println!("Parsing old schema failed with:");
                                        println!("{}", e);
                                    }
                                }
                            } else {
                                println!("Found no old version. Create {}.bs to see a changeset between the two versions", filename - 1);
                            }
                        }
                        Err(e) => {
                            println!("Failed to parse {}.bs with stacktrace:", filename);
                            println!("{}", e);
                        }
                    }
                }
                _ => (),
            },
            Err(e) => println!("watch error: {:?}", e),
        };
    }
}

pub type ExampleResult<T> = Result<T, ExampleError>;
#[derive(Error, Debug)]
pub enum ExampleError {
    #[error(transparent)]
    BUILDError(#[from] BUILDScriptError),
    #[error(transparent)]
    NotifyError(#[from] notify::Error),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

/// Continously parse all schemas in the versions folder
///
/// Whenever there exist two schemas with consequtive numbers. i.e 1.bs and 2.bs
/// A Changeset is made between them
fn main() -> ExampleResult<()> {
    let root_path_s = env::var("CARGO_MANIFEST_DIR").unwrap();
    let root_path = Path::new(&root_path_s);
    let test_dir = root_path.join(Path::new("examples/versions/"));
    if !test_dir.exists() {
        create_dir(&test_dir)?;
    }

    println!(
        "Listening for changes to files in {}",
        test_dir.to_str().unwrap()
    );
    // Automatically select the best implementation for your platform.
    let a = version_folder_listener();
    let mut watcher = notify::recommended_watcher(a)?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(&test_dir, RecursiveMode::NonRecursive)?;

    loop {
        sleep(Duration::from_millis(1000000));
    }
}
