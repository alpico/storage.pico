//! Disk usage for an ext4 filesystem.

use al_crunch_pool::{execute, Options, Sender};
use al_mmap::Mmap;
use ap_storage::Error;
use ap_storage_ext4_ro::{Ext4Fs, File};
use ap_storage_memory::ReadSlice;
use clap::Parser;
use std::rc::Rc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File to benchmark.
    #[arg(short, long)]
    file: String,

    /// Direct acccess.
    #[arg(short, long, default_value_t = false)]
    no_direct: bool,

    /// Leaf optimization
    #[arg(short, long, default_value_t = false)]
    leaf_optimization: bool,

    /// Number of parallel threads.
    #[arg(short, long)]
    threads: Option<usize>,

    /// Number of slots in the queue per thread.
    #[arg(short, long, default_value_t = 8)]
    slots: usize,

    /// Number of repeats.
    #[arg(short, long, default_value_t = 1)]
    repeat: usize,
}

pub struct WorkerState {
    fs: Rc<Ext4Fs<'static>>,
    size: u64,
    count: usize,
}

fn visit(sender: &Sender<WorkerState>, nr: u64, worker: &mut WorkerState) {
    let fs = Rc::clone(&worker.fs);
    let dir = File::new(&fs, nr).unwrap();
    let Some(mut iter) = dir.dir() else { return };

    // skip own and parent directories
    let _ = iter.next(&mut []);
    let _ = iter.next(&mut []);
    while let Ok(entry) = iter.next(&mut []) {
        if entry.name_len() == 0 {
            continue;
        }
        worker.count += 1;

        let Ok(child) = File::new(&fs, entry.inode()) else {
            continue;
        };
        worker.size += child.size();

        if entry.is_dir() {
            let sender2 = sender.clone();
            sender.send(worker, move |state| {
                visit(&sender2, entry.inode(), state);
            });
        }
    }
}

fn main() -> Result<(), Error> {
    let args = Args::parse();
    let mmap = Mmap::new(&args.file, !args.no_direct, 0, 0)?;
    let disk = ReadSlice(mmap.0);
    let disk: &(dyn ap_storage::Read + Sync) = &disk;

    let options = Options::default()
        .one_is_zero()
        .io_bound()
        .threads(args.threads)
        .slots(args.slots);

    // a function to produce the state for every worker
    let make_state = |_| {
        WorkerState {
            // XXX we don't handle the lifetimes correctly
            fs: Rc::new(
                Ext4Fs::new(unsafe { std::mem::transmute(disk) }, args.leaf_optimization).unwrap(),
            ),
            size: 0,
            count: 0,
        }
    };

    // the bounded Job queue
    for _i in 0..args.repeat {
        let state = execute(
            options.clone(),
            make_state,
            |state| (state.count, state.size),
            |sender| {
                let mut state = make_state(0);
                visit(sender, 2, &mut state);
                (state.count, state.size)
            },
            |mut x, y| {
                x.0 += y.0;
                x.1 += y.1;
                x
            },
        );
        println!("{} {} {}", args.file, state.0, state.1);
    }
    Ok(())
}
