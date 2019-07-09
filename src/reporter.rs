use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

/// Used to report progress about rendering
pub trait Reporter {
    fn new(pixels: u64) -> Self;
    fn report_finished_pixels(&self, finished: u64);
}

/// A low-overhead progress reporter with rich progress bar output
pub struct RenderProgress {
    thread_handle: Option<JoinHandle<()>>,
    stop: Arc<AtomicBool>,
    pixels_completed: Arc<AtomicU64>,
}

impl Reporter for RenderProgress {
    fn new(pixels: u64) -> Self {
        let pixels_completed = Arc::new(AtomicU64::default());
        let stop = Arc::new(AtomicBool::default());

        // Spawns a thread that periodically updates the progress bar without interrupting
        // the rest of the processing
        let pixels_completed_t = pixels_completed.clone();
        let stop_t = stop.clone();
        let thread_handle = thread::spawn(move || {
            // Disable progress bar for CI
            let progress = match env::var("CI") {
                Ok(ref val) if val == "true" => ProgressBar::hidden(),
                _ => {
                    let progress = ProgressBar::new(pixels);
                    progress.set_style(ProgressStyle::default_bar()
                        .template("[{elapsed_precise}] {wide_bar:.cyan/blue} {percent}% (eta: {eta})"));
                    progress
                },
            };

            while !stop_t.load(Ordering::SeqCst) {
                progress.set_position(pixels_completed_t.load(Ordering::SeqCst));

                thread::sleep(Duration::from_millis(100));
            }

            progress.finish_and_clear();
        });

        Self {
            thread_handle: Some(thread_handle),
            stop,
            pixels_completed,
        }
    }

    fn report_finished_pixels(&self, finished: u64) {
        // Trying to keep this as cheap as possible to not affect performance
        self.pixels_completed.fetch_add(finished, Ordering::SeqCst);
    }
}

impl Drop for RenderProgress {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.thread_handle.take().unwrap().join().unwrap();
    }
}

/// A zero-overhead reporter that does not actually produce any output or do any operations
/// whatsoever. This is meant to be used when performance is really critical and progress
/// does not need to be reported.
pub struct NullProgress;

impl Reporter for NullProgress {
    fn new(_pixels: u64) -> Self {
        Self
    }

    fn report_finished_pixels(&self, _finished: u64) {}
}
