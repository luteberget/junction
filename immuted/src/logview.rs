use backend_glfw::imgui::*;
use const_cstr::*;
use crate::ui;
use std::collections::VecDeque;
use log::*;
use std::sync::Arc;
use std::sync::Mutex;

pub fn view_log(popen :&mut bool, logstring :&Arc<Mutex<VecDeque<u8>>>) {
    unsafe {
        igBegin(const_cstr!("Log").as_ptr(), popen as _, 0 as _);
        igPushTextWrapPos(0.0);

        ui::show_text("Log:");
        {
        let buf = logstring.lock().unwrap();
        let (s1,s2) = buf.as_slices();
        let begin = s1.as_ptr() as *const i8;
        let end = begin.offset(s1.len() as isize);
        igTextUnformatted(begin,end);

        igSameLine(0.0,-1.0);

        let begin = s2.as_ptr() as *const i8;
        let end = begin.offset(s2.len() as isize);
        igTextUnformatted(begin,end);
        }

        igPopTextWrapPos();
        igEnd();
    }

}

pub struct StringLogger {
    level :LevelFilter,
    log :Arc<Mutex<VecDeque<u8>>>,
}

impl StringLogger {
    pub fn init(log_level :LevelFilter) -> Result<Arc<Mutex<VecDeque<u8>>>, SetLoggerError> {
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
