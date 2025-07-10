use std::fmt;
use tracing::debug;


pub fn display_info<T: fmt::Display>(info_title: &str, sub_title: bool, content: T) {
    let info_title_len = info_title.len();

    if info_title_len == 0 {
        debug!("{}\n", content);
        return;
    }

    if sub_title {
        const DASH_COUNT: usize = 32;
        let left_dashes = "-".repeat(DASH_COUNT);
        let right_dashes = "-".repeat(DASH_COUNT);
        debug!(
            "\n{} {} {}\n{}\n",
            left_dashes, info_title, right_dashes, content
        );
    } else {
        let frame_bound = "-".repeat(info_title_len);
        debug!(
            "\n{}\n{}\n{}\n{}\n",
            frame_bound, info_title, frame_bound, content
        );
    }
}