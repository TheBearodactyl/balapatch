use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use once_cell::sync::Lazy;

pub static GLOBAL_MP: Lazy<MultiProgress> = Lazy::new(MultiProgress::new);

pub fn create_spinner(message: &'static str) -> ProgressBar {
    let pb = GLOBAL_MP.add(ProgressBar::new_spinner());
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["◜", "◠", "◝", "◞", "◡", "◟"])
            .template("{spinner:.cyan} {msg}")
            .expect("fuck"),
    );
    pb.set_message(message);
    pb
}

pub fn create_bytes_progress(message: &'static str, total: u64) -> ProgressBar {
    let pb = GLOBAL_MP.add(ProgressBar::new(total));
    pb.set_style(
        ProgressStyle::default_bar()
            .progress_chars("##-")
            .template("{msg}\n{bar:40.cyan/blue} {bytes}/{total_bytes} ({eta})")
            .expect("fuck")
    );
    pb.set_message(message);
    pb
}
