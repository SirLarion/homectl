use log::{Record, Level, Metadata, SetLoggerError, LevelFilter};

#[derive(Debug)]
struct Logger {
  level: Level,
}

impl log::Log for Logger {
  fn enabled(&self, metadata: &Metadata) -> bool {
    metadata.level() <= self.level
  }

  fn log(&self, record: &Record) {
    if self.enabled(record.metadata()) {
      println!("[{}]: {}", record.level(), record.args());
    }
  }

  fn flush(&self) {}
}

pub struct LoggerFlags {
  pub verbose: bool,
  pub debug: bool,
}

pub fn init(flags: LoggerFlags) -> Result<(), SetLoggerError> {
  let logger = match flags {
    LoggerFlags { debug: true, .. } => Logger { level: Level::Debug },
    LoggerFlags { verbose: true, .. } => Logger { level: Level::Warn },
    _ => Logger { level: Level::Info }
  }; 
  log::set_boxed_logger(Box::new(logger))
    .map(|()| log::set_max_level(LevelFilter::Debug))
}
