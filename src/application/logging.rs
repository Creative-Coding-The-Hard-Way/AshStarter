use {
    anyhow::Result,
    flexi_logger::{
        DeferredNow, Duplicate, FileSpec, Logger, LoggerHandle, Record,
        WriteMode,
    },
    regex::Regex,
    std::{fmt::Write as FmtWrite, sync::Once},
    textwrap::{termwidth, Options},
};

/// A global handle to the initialized flexi_logger.
///
/// This gets setup on the first call to setup_logger().
static mut LOGGER_HANDLE: Option<LoggerHandle> = None;

/// Used to synchronize access to LOGGER_HANDLE.
static INIT: Once = Once::new();

//::new(r"(┃)(.*)$").unwrap();
static mut LAST_NEWLINE_DELIM_MACHER: Option<Regex> = None;

/// Setup pretty console and file logging.
pub fn setup() {
    INIT.call_once(|| {
        let handle = Logger::try_with_env_or_str("trace")
            .unwrap()
            .log_to_file(FileSpec::default().directory("logs"))
            .format(multiline_format)
            .duplicate_to_stdout(Duplicate::Debug)
            .write_mode(WriteMode::Async)
            .start()
            .expect("Unable to start the logger!");
        let matcher = Regex::new(r"(┃)(.*)$").unwrap();
        unsafe {
            LOGGER_HANDLE = Some(handle);
            LAST_NEWLINE_DELIM_MACHER = Some(matcher)
        };
    });
}

/// A multiline log format for flexi_logger.
///
/// Logs are automatically wrapped at terminal width and prefixed with unicode
/// so it's easy to tell where a big log statement begins and ends.
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

    let wrapped = textwrap::fill(&full_line, wrap_options);
    let formatted = unsafe {
        // Safe because the delimeter is setup once when the logger is first
        // created.
        LAST_NEWLINE_DELIM_MACHER
            .as_ref()
            .unwrap()
            .replace(&wrapped, "┗$2")
    };

    writeln!(w, "{}", formatted)
}
