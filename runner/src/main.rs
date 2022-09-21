use std::process::Command;

use clap::Parser;

fn main() {
  let cli = Args::parse();

  build_frontend(&cli);
  copy_files_to_webroot();
  build_backend(&cli);

  if cli.run {
    run_backend(&cli);
  }
}

fn build_frontend(args: &Args) {
  let current_dir = std::env::current_dir().unwrap();
  let current_dir = current_dir.to_string_lossy();

  let mut command = std::process::Command::new("trunk");
  command.arg("build").current_dir(format!("{current_dir}/frontend"));

  // if the backend is built in release mode, we also want to build the frontend in release mode
  if args.optimize {
    command.arg("--release");
  }

  // spawn and verify, that the command ran to completion. Otherwise panic, because the frontend couldn't be built and there is no use continuing.
  command.spawn().unwrap().wait().unwrap();
}

//TODO: automatically dereference cargo project root?
fn copy_files_to_webroot() {
  // clear webroot so we do not clutter it and create future problems. This might as well fail, and we cannot do anything about it, so any failure will be silently ignored:
  let current_dir = std::env::current_dir().unwrap();
  let current_dir = current_dir.to_string_lossy();
  let _ = std::fs::remove_dir_all(format!("{current_dir}/backend/web_root"));
  std::fs::create_dir(format!("{current_dir}/backend/web_root")).unwrap();

  std::fs::read_dir(format!("{current_dir}/frontend/dist"))
    .unwrap()
    .for_each(|file| {
      let file = file.unwrap();
      println!("{}", file.path().to_string_lossy());

      std::fs::copy(
        file.path(),
        format!("{current_dir}/backend/web_root/{}", file.file_name().to_string_lossy()),
      )
      .unwrap();
    });

  std::fs::copy(
    format!("{current_dir}/frontend/assets/favicon.svg"),
    format! {"{current_dir}/backend/web_root/favicon.svg"},
  )
  .unwrap();
}

fn build_backend(args: &Args) {
  let mut command = Command::new("cargo");

  command.args(["build", "--features", "serve_frontend", "--package", "backend"]);

  if args.optimize {
    command.arg("--release");
  }
  command.spawn().unwrap().wait().unwrap();
}

fn run_backend(args: &Args) {
  let current_dir = std::env::current_dir().unwrap();
  let current_dir = current_dir.to_string_lossy();
  let mut command = Command::new("cargo");
  command
    .args(["run", "--features", "serve_frontend"])
    .env("RUST_LOG", "debug")
    .current_dir(format!("{current_dir}/backend/"));

  if args.optimize {
    command.arg("--release");
  }

  command.spawn().unwrap().wait().unwrap();
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
  /// Name of the person to greet
  #[clap(short, long, value_parser)]
  optimize: bool,

  #[clap(short, long, value_parser)]
  run: bool,
}
