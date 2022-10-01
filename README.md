# Dependencies
The following applications are required in addition to rust to facilitate smooth development:

 * trunk
 * bower
   >if you wish not to use bower, you may manually place a normalize.css deployment in frontend/bower_components/normalize.css
# Development
In order to run the application for development, it is recommended to use the bundled runscript like so:

`cargo run -p runner -- [-o] [-r]`

Run `cargo run -p runner -- --help` for more information. You may also locally symlink the generated runner-program, if the long command bothers you: 
```
cargo build -p runner --release
ln -s target/release/runner ./run
```
And then repeatedly:
```
./run [-o] -[-r]
```

This runscript is required to mitigate issues regarding to CORS handling (as opposed to serving with trunk) and requires no external dependencies like webservers. The frontend will be served by the backend under the `/dev/` URI-prefix.
The runscript is only support on *nix operating systems.

# MSRV
The minimum supported rust version is `nightly` due to reliance on the `Try` trait to make code more ergonomic. You may need rustc v1.65 or later.