use crate::document::Document;
use crate::config::Config;
use crate::gui::logview::LogStore;


// app strucutre
//  + cross-cutting concerns 
//    - thread pool
//    - log
//    - handle to set 

pub struct App {
    pub document :Document,
    pub config :Config,
    pub log :LogStore,
    pub windows: Windows,
    pub background_jobs :BackgroundJobs,
}

#[derive(Clone)]
pub struct BackgroundJobs(threadpool::ThreadPool);

impl BackgroundJobs {
    pub fn new() -> Self { BackgroundJobs(threadpool::ThreadPool::new(2)) }

    pub fn execute(&mut self, job: impl FnOnce() + Send + 'static) {
        self.0.execute(job)
    }
}


pub struct Windows {
    pub debug: bool,
    pub log: bool,
    pub quit: bool,
    // ...
}

impl Windows {
    pub fn closed() -> Self {
        Windows {
            debug: false,
            log: false,
            quit: false,
            // ...
        }
    }
}

pub trait BackgroundUpdates {
    fn check(&mut self);
}

pub trait UpdateTime {
    fn advance(&mut self, dt :f64);
}


