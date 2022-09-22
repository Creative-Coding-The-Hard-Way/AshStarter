mod pretty_list;

use std::fmt::Write as FmtWrite;

use anyhow::Result;
use flexi_logger::{DeferredNow, Logger, Record};
use textwrap::{termwidth, Options};

pub use self::pretty_list::PrettyList;

/// Setup console logging for this application.
pub fn setup() -> Result<(), anyhow::Error> {
    Logger::with_env_or_str("info")
        .format(multiline_format)
        .start()?;

    log::info!(
        "Adjust the log level by setting RUST_LOG. By default RUST_LOG=info"
    );

    Ok(())
}

/// An opinionated formatting function for flexi_logger which automatically
/// wraps content to the terminal width.
pub fn multiline_format(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    let size = termwidth().min(74);
    let wrap_options = Options::new(size)
        .initial_indent("┏ ")
        .subsequent_indent("┃ ");

    let mut full_line = String::new();
    writeln!(
        full_line,
        "{} [{}] [{}:{}]",
        record.level(),
        now.now().format("%H:%M:%S%.6f"),
        record.file().unwrap_or("<unnamed>"),
        record.line().unwrap_or(0),
    )
    .expect("unable to format first log line");

    write!(&mut full_line, "{}", &record.args())
        .expect("unable to format log!");

    writeln!(w, "{}", textwrap::fill(&full_line, wrap_options))
}
