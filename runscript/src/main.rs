use std::env::current_dir;
use std::fs::{File, FileType};
use std::io::Stdout;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

use clap::Parser;
use nix::libc::{c_int, kill, sigaction};
use nix::sys::signal::{SigAction, SigHandler, Signal};
use nix::unistd::Pid;

static CHILD_PIDS: Mutex<Vec<u32>> = std::sync::Mutex::new(std::vec::Vec::new());

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
  /// Turn debugging information on
  #[arg(short, long, default_value_t = false)]
  self_debug: bool,

  /// Turn debugging information on
  #[arg(short, long, default_value_t = false)]
  debug_backend: bool,

  // log all outputs to a file, not just the backend output
  #[arg(short, long, default_value_t = false)]
  log_to_file: bool,
}
fn main() {
  let cli = Cli::parse();
  let cwd = std::env::current_dir().unwrap().canonicalize().unwrap();
  let cargo_root = find_cargo_root(cwd).unwrap();

  if cli.self_debug {
    println!("Found Cargo root at {}", cargo_root.display());
  }

  {
    let mut write_lock = CHILD_PIDS.lock().unwrap();
    write_lock.push(backend(&cargo_root));
    write_lock.push(tailwind_watcher(&cargo_root));
    write_lock.push(frontend(&cargo_root));
  }

  let sig_action = SigAction::new(
    SigHandler::Handler(handle_signal),
    nix::sys::signal::SaFlags::empty(),
    nix::sys::signal::SigSet::empty(),
  );

  unsafe {
    nix::sys::signal::sigaction(nix::sys::signal::Signal::SIGINT, &sig_action).unwrap();
  }
}

fn tailwind_watcher(cargo_root: &Path) -> u32 {
  // run tailwind watcher through bash
  let tailwind_watcher_log_file = File::create("./tailwind_watcher.log").unwrap();
  let frontend_root = cargo_root.join("./frontend");
  let command = Command::new("bash")
    .arg(frontend_root.join("./tailwind_watcher.sh"))
    .current_dir(frontend_root)
    .stdout(Stdio::from(tailwind_watcher_log_file))
    .spawn()
    .unwrap();
  let id = command.id();

  monitor_child(command, "tailwind_watcher");

  id
}

fn frontend(cargo_root: &Path) -> u32 {
  let frontend_root = cargo_root.join("./frontend");
  let frontend_watcher = Command::new("bash")
    .arg(frontend_root.join("./watch_desktop.sh"))
    .current_dir(frontend_root)
    .spawn()
    .unwrap();

  let id = frontend_watcher.id();

  monitor_child(frontend_watcher, "frontend_watcher");

  id
}

fn monitor_child(mut child: Child, child_name: &'static str) {
  std::thread::spawn(move || {
    if !child.wait().unwrap().success() {
      panic!("{child_name} stopped unexpectedly")
    }
  });
}

fn backend(cargo_root: &Path) -> u32 {
  let build_log_file = File::create("./builds.log").unwrap();
  let mut backend_build = Command::new("cargo")
    .args([
      "build",
      "--package",
      "backend",
      "--release",
      "--features",
      "serve_frontend",
    ])
    .stdout(Stdio::from(build_log_file))
    .spawn()
    .unwrap();

  // wait for build to complete and panic if it doesnt
  if !backend_build.wait().unwrap().success() {
    panic!("Backend failed to build")
  }
  let backend_log_file = File::create("./backend.log").unwrap();
  let backend_process = Command::new(cargo_root.join("./target/release/backend"))
    .current_dir(cargo_root.join("./backend"))
    .stdout(Stdio::from(backend_log_file))
    .spawn()
    .unwrap();

  let id = backend_process.id();

  monitor_child(backend_process, "backend");

  id
}

fn find_cargo_root(mut cwd: PathBuf) -> Option<PathBuf> {
  let mut cwd_read_dir = std::fs::read_dir(&cwd).unwrap();
  let mut found = false;
  // we should find Cargo.lock at the root of a Cargo workspace/standalone project
  // if we haven't found it we keep going up the file tree
  while let None = cwd_read_dir.find(|entry| {
    if let Ok(entry) = entry {
      let res = entry.file_name() == "Cargo.lock";
      if res {
        found = true;
      }
      res
    } else {
      false
    }
  }) {
    cwd.pop();
    cwd_read_dir = std::fs::read_dir(&cwd).unwrap();
  }

  if found {
    Some(cwd)
  } else {
    None
  }
}

extern "C" fn handle_signal(signal: c_int) {
  unsafe {
    let pids = CHILD_PIDS.lock().unwrap();
    for pid in &*pids {
      // Send the SIGINT signal to each child process
      let _ = kill(*pid as i32, Signal::SIGINT as c_int);
    }
  }
}
