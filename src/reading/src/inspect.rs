/* -----------------------------------------------------------------------------------------------
 * printing methods
 * -----------------------------------------------------------------------------------------------
 */
// use differential_dataflow::difference::Abelian;
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
use std::time::Duration;
use timely::dataflow::Scope;
use timely::order::TotalOrder;
use tracing::{debug, error, info};

use crate::rel::Rel;
use crate::semiring_one;

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
                std::fs::create_dir_all(parent).expect("Can not create output directory");
            }

            // Open file for writing
            let file = File::create(path).expect(&format!("Can not create output file: {}", path));
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

    rel.threshold_semigroup(move |_, _, old| old.is_none().then_some(semiring_one()))
        .lift(|_| Some(((), 1 as i32)))
        .map(|_| ())
        .consolidate()
        .inspect(move |x| info!("{}: {:?}", prefix, x));
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
    rel.threshold_semigroup(move |_, _, old| old.is_none().then_some(semiring_one()))
        .lift(|x| Some((x, 1 as i32)))
        .inspect(move |(data, time, delta)| debug!("{}: ({}, {:?}, {})", name, data, time, delta));
    // use std::fmt::Display for D (i.e. Row)
}

/// Write relation size
fn writesize<G, D, R>(rel: &Collection<G, D, R>, name: &str, file_path: &str)
where
    G: Scope,
    G::Timestamp: Lattice + TotalOrder,
    D: ExchangeData + Hashable,
    R: Semigroup + ExchangeData,
{
    let file_handle = get_file_handle(&file_path);
    let file_path = file_path.to_string();
    let name = name.to_string();

    rel.threshold_semigroup(move |_, _, old| old.is_none().then_some(semiring_one()))
        .lift(|_| Some(((), 1 as i32)))
        .map(|_| ())
        .consolidate()
        .inspect({
            move |x| {
                let mut file = file_handle.lock().unwrap();
                writeln!(file, "{}: {:?}", name, x)
                    .expect(&format!("Can not write size: {}", file_path));
            }
        });
}

/// Flush relation data to a file
fn write<G, D, R>(rel: &Collection<G, D, R>, file_path: &str, worker_id: usize)
where
    G: Scope,
    G::Timestamp: Lattice + TotalOrder,
    D: ExchangeData + Hashable + std::fmt::Display,
    R: Semigroup + ExchangeData,
{
    let path = format!("{}{}", file_path, worker_id);
    let file_handle = get_file_handle(&path);

    rel.threshold_semigroup(move |_, _, old| old.is_none().then_some(semiring_one()))
        .lift(|x| Some((x, 1 as i32)))
        .inspect(move |(data, _time, _delta)| {
            let mut file = file_handle.lock().unwrap();
            writeln!(file, "{}", data).expect(&format!("Can not write: {}", path));
        });

    // alternative (faster)
    // .inspect_batch(move |_batch_time, rows| {
    //     let mut file = file_handle.lock().unwrap();
    //     let batch_str = rows.iter().map(|(data, _, _)| format!("{}", data)).collect::<Vec<_>>().join("\n");
    //     writeln!(file, "{}", batch_str).expect(&format!("Can not write to file: {}", path));
    // });
}

/// Prints the content of a relation with any arity
pub fn print_generic<G>(rel: &Rel<G>, name: &str)
where
    G: Scope,
    G::Timestamp: Lattice + TotalOrder,
{
    if rel.is_fat() {
        print(rel.rel_fat(), name)
    } else {
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
            _ => unreachable!("arity {} should be handled by fixed-size variants", arity),
        }
    }
}

/// Prints the size of a relation with any arity
pub fn printsize_generic<G>(rel: &Rel<G>, name: &str, is_recursive: bool)
where
    G: Scope,
    G::Timestamp: Lattice + TotalOrder,
{
    if rel.is_fat() {
        printsize(rel.rel_fat(), name, is_recursive)
    } else {
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
            _ => unreachable!("arity {} should be handled by fixed-size variants", arity),
        }
    }
}

/// Writes a relation with any arity to a file
pub fn write_generic<G>(rel: &Rel<G>, file_path: &str, worker_id: usize)
where
    G: Scope,
    G::Timestamp: Lattice + TotalOrder,
{
    if rel.is_fat() {
        write(rel.rel_fat(), file_path, worker_id)
    } else {
        let arity = rel.arity();
        match arity {
            1 => write(rel.rel_1(), file_path, worker_id),
            2 => write(rel.rel_2(), file_path, worker_id),
            3 => write(rel.rel_3(), file_path, worker_id),
            4 => write(rel.rel_4(), file_path, worker_id),
            5 => write(rel.rel_5(), file_path, worker_id),
            6 => write(rel.rel_6(), file_path, worker_id),
            7 => write(rel.rel_7(), file_path, worker_id),
            8 => write(rel.rel_8(), file_path, worker_id),
            _ => unreachable!("arity {} should be handled by fixed-size variants", arity),
        }
    }
}

/// Writes a relation size with any arity to a file
pub fn writesize_generic<G>(rel: &Rel<G>, name: &str, file_path: &str)
where
    G: Scope,
    G::Timestamp: Lattice + TotalOrder,
{
    if rel.is_fat() {
        writesize(rel.rel_fat(), name, file_path)
    } else {
        let arity = rel.arity();
        match arity {
            1 => writesize(rel.rel_1(), name, file_path),
            2 => writesize(rel.rel_2(), name, file_path),
            3 => writesize(rel.rel_3(), name, file_path),
            4 => writesize(rel.rel_4(), name, file_path),
            5 => writesize(rel.rel_5(), name, file_path),
            6 => writesize(rel.rel_6(), name, file_path),
            7 => writesize(rel.rel_7(), name, file_path),
            8 => writesize(rel.rel_8(), name, file_path),
            _ => unreachable!("arity {} should be handled by fixed-size variants", arity),
        }
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
    let file_handle = get_file_handle(&format!("{}", output_path));

    // Read and concatenate all existing worker files
    let merged_content = (0..worker_count)
        .filter_map(|worker_id| {
            let part_path = format!("{}{}", output_path, worker_id);
            match read_to_string(&part_path) {
                Ok(content) => Some(content),
                Err(_) => {
                    if worker_id == 0 {
                        // log a warning
                        debug!("Warning: missing or unreadable file {}", part_path);
                    }
                    None
                }
            }
        })
        .collect::<Vec<_>>()
        .join("");

    let mut file = file_handle.lock().unwrap();
    // Dump the merged content into the main output file
    if let Err(e) = file.write_all(merged_content.as_bytes()) {
        error!("Error to write merged file {}: {}", output_path, e);
    }

    // Attempt to remove all partial files, ignore failure
    for worker_id in 0..worker_count {
        let part_path = format!("{}{}", output_path, worker_id);
        let _ = remove_file(&part_path);
    }
}

/// Records the elapsed time (in seconds) to a file.
///
/// Appends the elapsed time to the specified file.  
/// Automatically creates directories and reuses file handles.
///
/// # Arguments
///
/// - `file_path`: The path to the file.
/// - `time_elapsed`: Time elapsed in seconds (f64).
pub fn record_time(file_path: &str, time_elapsed: Duration) {
    let file_handle = get_file_handle(file_path);
    let seconds = time_elapsed.as_secs_f64();
    let mut file = file_handle.lock().unwrap();
    writeln!(file, "{:.6} seconds elapsed", seconds)
        .expect(&format!("Can not write time to file: {}", file_path));
}

/// Closes all open file handles
///
/// Call this function at the end of the program to ensure all files are properly closed.
pub fn close_all_files() {
    FILE_HANDLES.with(|handles| {
        let mut handles_ref = handles.borrow_mut();
        handles_ref.clear(); // Dropping all Arc<Mutex<File>> will close the files
    });
    debug!("All output files closed");
}
