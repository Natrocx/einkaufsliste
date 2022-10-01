use std::cell::RefCell;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use clap::Parser;
use log::debug;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use notify_debouncer_mini::{new_debouncer,  DebouncedEvent};

fn main() {
  let current_dir = std::env::current_dir().unwrap();
  let project_root = PathBuf::from(find_project_root(&current_dir).unwrap());

  let cli = Cli::parse();
  build_frontend(&cli, &project_root);
  copy_files_to_webroot(&project_root);
  build_backend(&cli);

  if cli.run {
    // scoped threads to avoid lifetime issues and unnecessary reference counting; also automatically stops threads when runscript is stopped
    std::thread::scope(|cx| {
      cx.spawn(|| {
        watch_frontend(&cli, &project_root);
      });
      cx.spawn(|| {
        run_and_watch_backend(&cli, &project_root);
      });
    });
  }
}

fn watch_frontend(cli: &Cli, project_root: &Path) {
  let (tx, fe_rx) = std::sync::mpsc::channel();

  let mut fe_debouncer = new_debouncer(Duration::from_secs(2), None, tx).unwrap();

  fe_debouncer
    .watcher()
    .watch(
      Path::new(&format!("{}/frontend", project_root.display())),
      notify::RecursiveMode::Recursive,
    )
    .unwrap();

  for res in fe_rx {
    match res {
      Err(e) => panic!("Failed to receive fs events on frontend: {e:?}"),
      Ok(event) if filter_frontend_events(&event, project_root) => {
        build_frontend(cli, project_root);
        copy_files_to_webroot(project_root);
      }
      Ok(_) => {
        // do not rebuild frontend
      }
    }
  }
}

fn run_and_watch_backend(cli: &Cli, project_root: &Path) {
  let (tx, rx) = std::sync::mpsc::channel();

  let mut be_debouncer = new_debouncer(Duration::from_secs(2), None, tx).unwrap();

  // run backend and save handle in a thread-safe, async-safe way
  let be_handle = Arc::new(Mutex::new(RefCell::new(run_backend(cli, project_root).unwrap())));

  be_debouncer
    .watcher()
    .watch(
      Path::new(&format!("{}/backend", project_root.display())),
      notify::RecursiveMode::Recursive,
    )
    .unwrap();

  for res in rx {
    match res {
      Err(e) => panic!("Failed to receive fs events on backend: {e:?}"),
      Ok(event) if filter_backend_events(&event, project_root) => {
        let be_handle = be_handle.lock().unwrap();
        stop_backend(&be_handle.borrow());
        be_handle.replace(run_backend(cli, project_root).unwrap());
      }
      Ok(_) => {
        // do not restart backend
      }
    }
  }
}

fn filter_backend_events(event: &Vec<DebouncedEvent>, project_root: &Path) -> bool {
  debug!("Encountered fs watcher event: {event:?}");

  let web_root_path = PathBuf::from(format!("{}/backend/web_root", project_root.display()));
  let data_path = PathBuf::from(format!("{}/backend/data.sled", project_root.display()));

  // we do not restart the backend if all changes to the backends directory were inside the servers web_root
  !event.iter().all(|debounced_event| {
    debounced_event.path.starts_with(&web_root_path) || debounced_event.path.starts_with(&data_path)
  })
}

fn filter_frontend_events(event: &Vec<DebouncedEvent>, project_root: &Path) -> bool {
  let dist_path = PathBuf::from(format!("{}/frontend/dist/", project_root.display()));

  debug!("Encountered fs watcher event: {event:?}");
  !event
    .iter()
    .all(|debounced_event| debounced_event.path.starts_with(&dist_path))
}

fn stop_backend(child: &Child) {
  // ignore result - if the backend couldn't be terminated, it doesn't run
  let _ = signal::kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM);
}

fn find_project_root(path: &Path) -> std::io::Result<&Path> {
  let potential_lockfile = path.ancestors().find(|f| {
    if let Ok(mut read_dir) = std::fs::read_dir(f) {
      read_dir.any(|f| match f {
        Ok(ent) => ent.file_name() == "Cargo.lock",
        Err(_) => false,
      })
    } else {
      false
    }
  });
  if let Some(lock_file) = potential_lockfile {
    Ok(lock_file)
  } else {
    Err(std::io::Error::new(ErrorKind::NotFound, "No project found at path"))
  }
}

fn build_frontend(args: &Cli, project_root: &Path) {
  let mut command = std::process::Command::new("trunk");
  command
    .args(["build", "--features", "dev_router"])
    .current_dir(format!("{}/frontend", project_root.display()));

  // if the backend is built in release mode, we also want to build the frontend in release mode
  if args.optimize {
    command.arg("--release");
  }

  // spawn and verify, that the command ran to completion. Otherwise panic, because the frontend couldn't be built and there is no use continuing.
  command.spawn().unwrap().wait().unwrap();
}

fn copy_files_to_webroot(project_root: &Path) {
  let project_root = project_root.display();
  // clear webroot so we do not clutter it and create future problems. This might as well fail, and we cannot do anything about it, so any failure will be silently ignored:
  let _ = std::fs::remove_dir_all(format!("{project_root}/backend/web_root"));
  std::fs::create_dir(format!("{project_root}/backend/web_root")).unwrap();

  std::fs::read_dir(format!("{project_root}/frontend/dist"))
    .unwrap()
    .for_each(|file| {
      let file = file.unwrap();

      std::fs::copy(
        file.path(),
        format!("{project_root}/backend/web_root/{}", file.file_name().to_string_lossy()),
      )
      .unwrap();
    });

  std::fs::copy(
    format!("{project_root}/frontend/assets/favicon.svg"),
    format! {"{project_root}/backend/web_root/favicon.svg"},
  )
  .unwrap();
}

fn build_backend(args: &Cli) {
  let mut command = Command::new("cargo");

  command.args(["build", "--features", "serve_frontend", "--package", "backend"]);

  if args.optimize {
    command.arg("--release");
  }
  command.spawn().unwrap().wait().unwrap();
}

fn run_backend(args: &Cli, project_root: &Path) -> Result<std::process::Child, std::io::Error> {
  let mut command = Command::new("cargo");
  command
    .args(["run", "--features", "serve_frontend"])
    .env("RUST_LOG", "debug")
    .current_dir(format!("{}/backend/", project_root.display()));

  if args.optimize {
    command.arg("--release");
  }

  command.spawn()
}

/// Arguments to the runscript
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
  /// Name of the person to greet
  #[clap(short, long, value_parser)]
  optimize: bool,

  #[clap(short, long, value_parser)]
  run: bool,
}
