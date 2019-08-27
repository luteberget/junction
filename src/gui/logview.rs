use backend_glfw::imgui::*;
use const_cstr::*;
use std::collections::VecDeque;
use log::*;
use std::sync::Arc;
use std::sync::Mutex;
use crate::gui::widgets;

pub fn view_log(popen :&mut bool, logstring :&Arc<Mutex<VecDeque<u8>>>) {
    if !*popen { return; }

    unsafe {
        igBegin(const_cstr!("Log").as_ptr(), popen as _, 0 as _);
        widgets::show_text("Log:");
        let buf = logstring.lock().unwrap();
        let (s1,s2) = buf.as_slices();
        widgets::long_text(std::str::from_utf8_unchecked(s1));
        igSameLine(0.0,-1.0);
        widgets::long_text(std::str::from_utf8_unchecked(s2));
        igEnd();
    }

}

pub struct StringLogger {
    level :LevelFilter,
    log :Arc<Mutex<VecDeque<u8>>>,
}

pub type LogStore = Arc<Mutex<VecDeque<u8>>>;

impl StringLogger {
    pub fn init(log_level :LevelFilter) -> Result<LogStore, SetLoggerError> {
        let log = Arc::new(Mutex::new(VecDeque::new()));
        set_max_level(log_level.clone());
        set_boxed_logger(Box::new(Self::new(log_level,log.clone())))?;
        Ok(log)
    }

    pub fn new(log_level :LevelFilter, log :Arc<Mutex<VecDeque<u8>>>) -> Self {
        StringLogger {
            level: log_level,
            log: log,
        }
    }

}

impl Log for StringLogger {
    fn flush(&self) {}
    fn enabled(&self, metadata :&Metadata<'_>) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record :&Record<'_>) {
        if self.enabled(record.metadata()) {
            let mut buf = self.log.lock().unwrap();
            let max_len = 100_000;
            let target = if record.target().len() > 0 {
                record.target()
            } else {
                record.module_path().unwrap_or_default()
            };
            let statement = format!("{:<5} [{}] {}\n",
                                    record.level().to_string(),
                                    target,
                                    record.args());

            let trim = ((buf.len() as isize + statement.len() as isize) - max_len).max(0) as usize;
            drop(buf.drain(0..trim));
            buf.extend(statement.bytes());
        }
    }
}
