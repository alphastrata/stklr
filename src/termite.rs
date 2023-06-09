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
                record.target(),
                record.line().unwrap_or(0),
                message
            ))
        })
        .level(log::LevelFilter::Warn)
        .chain(std::io::stdout())
        .chain(fern::log_file(termite_path)?)
        .apply()?;

    info!("Logger setup complete.");
    Ok(())
}
