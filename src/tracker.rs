use std::{
    io::{self, Write},
    time::Instant,
};

use colored::{ColoredString, Colorize};

pub struct ProgressTracker {
    content_length: usize,

    pub total_read: usize,
    incremental_read: usize,

    percentage: f32,
    start: Instant,
    timer: Instant,
}

impl ProgressTracker {
    pub fn new(content_length: usize) -> Self {
        ProgressTracker {
            content_length,
            total_read: 0,
            incremental_read: 0,
            percentage: 0.0,
            start: Instant::now(),
            timer: Instant::now(),
        }
    }

    pub fn update(&mut self, read: usize) {
        self.total_read += read;
        self.percentage = self.total_read as f32 / self.content_length as f32 * 100.0;

        self.incremental_read += read;

        let incremental_time = self.timer.elapsed().as_secs_f32();

        if incremental_time > 1.0 {
            let kbs = self.incremental_read as f32 / incremental_time / 1000.0;

            self.display(kbs);

            self.timer = Instant::now();
            self.incremental_read = 0;
        }
    }

    fn estimated(&self) -> String {
        let rate = self.total_read as u64 / self.start.elapsed().as_secs();
        let remaining = self.content_length - self.total_read;
        let estimated_total = remaining as u64 / rate;

        let min = estimated_total / 60;
        let secs = estimated_total.rem_euclid(60);

        format!("{}min{}s", min, secs)
    }

    fn kbs_to_human_readable(kbs: f32) -> String {
        if kbs > 1000.0 {
            let kbs_string = format!("{:>5.1}", kbs / 1000.0).blue();
            format!("{} mb/s", kbs_string)
        } else {
            let kbs_string = format!("{:>5.1}", kbs).blue();
            format!("{} kb/s", kbs_string)
        }
    }

    fn elapsed_to_human_readable(&self) -> String {
        let secs = self.start.elapsed().as_secs();
        if secs < 60 {
            format!("{}s", secs)
        } else {
            let min = secs / 60;

            format!("{}min{}s", min, secs.rem_euclid(60))
        }
    }

    fn progress_bar(&self) -> ColoredString {
        let bar_length = 15.0;

        let current_index = ((self.percentage / 100.0) * bar_length).floor() as usize;

        let mut bar = "".to_owned();
        for idx in 0..bar_length as usize {
            if idx < current_index {
                bar.push('⣿');
            } else {
                bar.push('·');
            }
        }

        format!(
            "{:>5.1}%  [{}] {}",
            self.percentage,
            bar.green(),
            current_index
        )
        .bold()
    }

    fn display(&self, kbs: f32) {
        print!(
            "\r{} | {} | {} | estimated {}",
            self.progress_bar(),
            ProgressTracker::kbs_to_human_readable(kbs),
            self.elapsed_to_human_readable(),
            self.estimated()
        );

        io::stdout().flush().unwrap();
    }

    pub fn flush(&self) {
        let incremental_time = self.timer.elapsed().as_secs_f32();
        let kbs = self.incremental_read as f32 / incremental_time / 1000.0;
        self.display(kbs);
    }
}
