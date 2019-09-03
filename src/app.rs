use crate::document::Document;
use crate::config::Config;
use crate::gui::windows::logview::LogStore;
use crate::import;

pub struct App {
    pub document :Document,
    pub config :Config,
    pub log :LogStore,
    pub windows: Windows,
    pub background_jobs :BackgroundJobs,
    //    - TODO set window name
    //    - TODO font / font size?
}

#[derive(Clone)]
/// Wrapper for thread pool.
pub struct BackgroundJobs(threadpool::ThreadPool);

impl BackgroundJobs {
    pub fn new() -> Self { BackgroundJobs(threadpool::ThreadPool::new(2)) }

    /// Run the given function as a background job.
    pub fn execute(&mut self, job: impl FnOnce() + Send + 'static) {
        self.0.execute(job)
    }
}

pub struct Windows {
    pub config: bool,
    pub debug: bool,
    pub log: bool,
    pub quit: bool,
    pub vehicles: bool,
    pub diagram_split :Option<f32>,
    pub import_window :import::ImportWindow,
}

impl Windows {
    pub fn closed(bg :BackgroundJobs) -> Self {
        Windows {
            config :false,
            debug: false,
            log: false,
            quit: false,
            vehicles: false,
            diagram_split: None,
            import_window: import::ImportWindow::new(bg),
        }
    }
}

pub trait BackgroundUpdates {
    fn check(&mut self);
}

pub trait UpdateTime {
    fn advance(&mut self, dt :f64);
}


