#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

pub fn setup_logger() -> Result<(), fern::InitError> {
    let termite_path = format!("termite_{}.log", chrono::Local::now().format("%Y-%m-%d"));
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.level(),
                record.target(), // The file that spawned this entry into the log.
                record.line().unwrap_or(0), // The line number in said file.
                message // The message a developer has defined within the info!/error!/warn! macro.
            ))
        })
        .level(log::LevelFilter::Info) // NOTE: we may want to change this in prod.
        .chain(std::io::stdout()) // NOTE: we may wnat to remove this in prod, as we don't really
        .chain(fern::log_file(termite_path)?)
        .apply()?;

    info!("Logger setup complete.");
    Ok(())
}
