use std::io::Write;

pub fn init_logger(verbose: bool, quiet: bool) {
    let env = env_logger::Env::default().filter_or(
        env_logger::DEFAULT_FILTER_ENV,
        if quiet {
            "error"
        } else if verbose {
            "debug"
        } else {
            "info"
        },
    );
    let mut builder = env_logger::Builder::from_env(env);
    if !verbose {
        builder.format(|buf, record| writeln!(buf, "{}", record.args()));
    }
    builder.init();
}
