mod pretty_list;

use {
    anyhow::{Context, Result},
    flexi_logger::{
        DeferredNow, Duplicate, FileSpec, Logger, Record, WriteMode,
    },
    std::fmt::Write as FmtWrite,
    textwrap::{termwidth, Options},
};

pub use self::pretty_list::PrettyList;

/// Setup console logging for this application.
pub fn setup() -> Result<flexi_logger::LoggerHandle, anyhow::Error> {
    Logger::try_with_env_or_str("trace")?
        .log_to_file(FileSpec::default().directory("logs"))
        .format(multiline_format)
        .duplicate_to_stdout(Duplicate::Info)
        .write_mode(WriteMode::Async)
        .start()
        .context("Unable to create application logger")
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
