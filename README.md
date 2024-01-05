# Dependencies
The following applications are required in addition to rust to facilitate smooth development:

 * dx - Dioxus cli utility
  > install with `cargo install dioxus-cli`
 * tailwind toolchain

# Development
To run the backend you may use the runscript in the backend folder. This will also make the backend serve the frontend under `https://localhost:8443/dev/index.html`.

Running the frontend natively may be done using `bash watch_desktop.sh` inside the frontend directory.

# MSRV
The minimum supported rust version is `nightly` due to reliance on the `Try` trait to make code more ergonomic. You may need rustc v1.65 or later.