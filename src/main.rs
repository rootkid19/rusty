fn main() {
    if let Err(err) = rusty::cli::run() {
        eprintln!("{err:#}");
        std::process::exit(1);
    }
}
