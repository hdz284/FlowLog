//! File output and inspection utilities for differential dataflow relations
//!
//! This module provides functions for printing relation contents, displaying relation sizes,
//! and writing relations to files.

use differential_dataflow::difference::Present;
use differential_dataflow::difference::Semigroup;
use differential_dataflow::lattice::Lattice;
use differential_dataflow::operators::threshold::ThresholdTotal;
use differential_dataflow::{Collection, ExchangeData, Hashable};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{read_to_string, remove_file, File};
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use timely::dataflow::Scope;
use timely::order::TotalOrder;

use crate::rel::Rel;

// Thread-local storage for file handles to avoid repeatedly opening the same files
thread_local! {
    static FILE_HANDLES: RefCell<HashMap<String, Arc<Mutex<File>>>> = RefCell::new(HashMap::new());
}

/// Gets or creates a file handle for the specified path
///
/// Ensures each path has only one file handle and creates parent directories if needed.
fn get_file_handle(path: &str) -> Arc<Mutex<File>> {
    let path_str = path.to_string();

    FILE_HANDLES.with(|handles| {
        let mut handles_ref = handles.borrow_mut();

        if !handles_ref.contains_key(&path_str) {
            // Create directory if it doesn't exist
            if let Some(parent) = Path::new(path).parent() {
                std::fs::create_dir_all(parent).expect("Failed to create output directory");
            }

            // Open file for writing
            let file =
                File::create(path).expect(&format!("Failed to create output file: {}", path));
            handles_ref.insert(path_str.clone(), Arc::new(Mutex::new(file)));
        }

        Arc::clone(handles_ref.get(&path_str).unwrap())
    })
}

/// Prints the size of a relation (number of tuples)
fn printsize<G, D, R>(rel: &Collection<G, D, R>, name: &str, is_recursive: bool)
where
    G: Scope,
    G::Timestamp: Lattice + TotalOrder,
    D: ExchangeData + Hashable,
    R: Semigroup + ExchangeData,
{
    let prefix = if is_recursive {
        format!("Delta of (recursive) {}", name)
    } else {
        format!("Size of (non-recursive) {}", name)
    };

    rel.threshold_semigroup(move |_, _, old| old.is_none().then_some(Present {}))
        .expand(|_| Some(((), 1 as i32)))
        .map(|_| ())
        .consolidate()
        .inspect(move |x| println!("{}: {:?}", prefix, x));
}

/// Prints the content of a relation (all tuples)
fn print<G, D, R>(rel: &Collection<G, D, R>, name: &str)
where
    G: Scope,
    G::Timestamp: Lattice + TotalOrder,
    D: ExchangeData + Hashable + std::fmt::Display,
    R: Semigroup + ExchangeData,
{
    let name = name.to_owned();
    rel.threshold_semigroup(move |_, _, old| old.is_none().then_some(Present {}))
        .expand(|x| Some((x, 1 as i32)))
        .inspect(move |(data, time, delta)| println!("{}: ({}, {:?}, {})", name, data, time, delta));
}

/// Writes relation data to a file
fn write_to_file<G, D, R>(rel: &Collection<G, D, R>, name: &str, file_path: &str, worker_id: usize)
where
    G: Scope,
    G::Timestamp: Lattice + TotalOrder,
    D: ExchangeData + Hashable + std::fmt::Display,
    R: Semigroup + ExchangeData,
{
    let name = name.to_owned();
    let path = format!("{}{}", file_path, worker_id);
    let file_handle = get_file_handle(&path);

    println!("Writing relation {} to file {}", name, path);

    rel.threshold_semigroup(move |_, _, old| old.is_none().then_some(Present {}))
        .expand(|x| Some((x, 1 as i32)))
        .inspect(move |(data, _time, _delta)| {
            let mut file = file_handle.lock().unwrap();
            writeln!(file, "{}", data).expect(&format!("Failed to write to file: {}", path));
        });
}

/// Prints the content of a relation with any arity
pub fn print_generic<G>(rel: &Rel<G>, name: &str)
where
    G: Scope,
    G::Timestamp: Lattice + TotalOrder,
{
    let arity = rel.arity();
    match arity {
        1 => print(rel.rel_1(), name),
        2 => print(rel.rel_2(), name),
        3 => print(rel.rel_3(), name),
        4 => print(rel.rel_4(), name),
        5 => print(rel.rel_5(), name),
        6 => print(rel.rel_6(), name),
        7 => print(rel.rel_7(), name),
        8 => print(rel.rel_8(), name),
        9 => print(rel.rel_9(), name),
        10 => print(rel.rel_10(), name),
        _ => panic!("print_generic unimplemented for arity {}", arity),
    }
}

/// Prints the size of a relation with any arity
pub fn printsize_generic<G>(rel: &Rel<G>, name: &str, is_recursive: bool)
where
    G: Scope,
    G::Timestamp: Lattice + TotalOrder,
{
    let arity = rel.arity();
    match arity {
        1 => printsize(rel.rel_1(), name, is_recursive),
        2 => printsize(rel.rel_2(), name, is_recursive),
        3 => printsize(rel.rel_3(), name, is_recursive),
        4 => printsize(rel.rel_4(), name, is_recursive),
        5 => printsize(rel.rel_5(), name, is_recursive),
        6 => printsize(rel.rel_6(), name, is_recursive),
        7 => printsize(rel.rel_7(), name, is_recursive),
        8 => printsize(rel.rel_8(), name, is_recursive),
        9 => printsize(rel.rel_9(), name, is_recursive),
        10 => printsize(rel.rel_10(), name, is_recursive),
        _ => panic!("printsize_generic unimplemented for arity {}", arity),
    }
}

/// Writes a relation with any arity to a file
pub fn write_relation_to_file<G>(rel: &Rel<G>, name: &str, file_path: &str, worker_id: usize)
where
    G: Scope,
    G::Timestamp: Lattice + TotalOrder,
{
    let arity = rel.arity();
    match arity {
        1 => write_to_file(rel.rel_1(), name, file_path, worker_id),
        2 => write_to_file(rel.rel_2(), name, file_path, worker_id),
        3 => write_to_file(rel.rel_3(), name, file_path, worker_id),
        4 => write_to_file(rel.rel_4(), name, file_path, worker_id),
        5 => write_to_file(rel.rel_5(), name, file_path, worker_id),
        6 => write_to_file(rel.rel_6(), name, file_path, worker_id),
        7 => write_to_file(rel.rel_7(), name, file_path, worker_id),
        8 => write_to_file(rel.rel_8(), name, file_path, worker_id),
        9 => write_to_file(rel.rel_9(), name, file_path, worker_id),
        10 => write_to_file(rel.rel_10(), name, file_path, worker_id),
        _ => panic!("write_relation_to_file unimplemented for arity {}", arity),
    }
}

/// Merge CSV output files from multiple workers into a single file.
///
/// Each worker produces a file named `<relation_name>_<worker_id>.csv`.
/// This function merges them into `<relation_name>.csv` and deletes the partial files.
///
/// # Arguments
///
/// - `output_dir`: The directory containing worker partition files.
/// - `worker_count`: Number of workers (used to find all partial files).
pub fn merge_relation_partitions(output_path: &str, worker_count: usize) {
    let file_handle = get_file_handle(&output_path);

    // Read and concatenate all worker files
    let merged_content = (0..worker_count)
        .map(|worker_id| {
            let part_path = format!("{}{}", output_path, worker_id);
            read_to_string(&part_path).unwrap_or_else(|_| panic!("Failed to read {}", part_path))
        })
        .collect::<Vec<_>>()
        .join("");

    let mut file = file_handle.lock().unwrap();
    file.write_all(merged_content.as_bytes())
        .unwrap_or_else(|_| panic!("Failed to write merged file: {}", output_path));

    // Optionally clean up partial files
    for worker_id in 0..worker_count {
        let part_path = format!("{}{}", output_path, worker_id);
        remove_file(&part_path).unwrap_or_else(|_| panic!("Failed to remove {}", part_path));
    }
}

/// Closes all open file handles
///
/// Call this function at the end of the program to ensure all files are properly closed.
pub fn close_all_files() {
    FILE_HANDLES.with(|handles| {
        let mut handles_ref = handles.borrow_mut();
        handles_ref.clear(); // Dropping all Arc<Mutex<File>> will close the files
    });
    println!("All output files closed");
}