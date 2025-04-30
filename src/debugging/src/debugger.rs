use std::fmt;


pub fn display_info<T: fmt::Display>(info_title: &str, sub_title: bool, content: T, verbose: bool) {
    if verbose {
        let info_title_len = info_title.len();
        if info_title_len > 0 {
            if !sub_title {
                let frame_bound = "-".repeat(info_title_len);
                println!(
                    "{}\n{}\n{}\n{}",
                    frame_bound, info_title, frame_bound, content
                );
            } else {
                println!("-------------------------------- {} --------------------------------\n{}", info_title, content);
            }
        } else {
            println!("{}", content);
        }
        println!();
    }
}