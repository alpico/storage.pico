//! Disk usage for an ext4 filesystem.

use al_crunch_pool::{execute, PoolOptions, Sender};
use ap_storage::Error;
use ap_storage_ext4_ro::{Ext4Fs, File};
use ap_storage_linux::memdisk::MemDisk;
use clap::Parser;


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
    fs: Ext4Fs<'static>,
    size: u64,
    count: usize,
}

fn visit(sender: &Sender<WorkerState>, nr: u64, worker: &mut WorkerState) {
    let fs = worker.fs.clone();
    let dir = File::new(&fs, nr).unwrap();
    let Some(mut iter) = dir.dir() else { return };

    let mut buf = [0u8; 255];
    while let Ok(entry) = iter.next(&mut buf) {
        let name = &buf[..entry.name_len()];
        if entry.name_len() < 3 && (name == b"" || name == b"." || name == b"..") {
            continue;
        }
        worker.count += 1;

        let Ok(child) = dir.open(entry.inode()) else {
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
    let disk = MemDisk::new(&args.file, !args.no_direct)?;

    // XXX we don't handle the lifetimes correctly
    let x: &dyn ap_storage::Read = &disk;
    let fs = Ext4Fs::mount(unsafe { std::mem::transmute(x) }, args.leaf_optimization)?;

    let config = PoolOptions::default()
        .one_is_zero()
        .io_bound()
        .threads(args.threads)
        .slots(args.slots);

    for _i in 0..args.repeat {
        // a function to produce the state for every worker
        let make_state = |_| WorkerState {
            fs: fs.clone(),
            size: 0,
            count: 0,
        };

        let state = execute(
            config.clone(),
            make_state,
            |sender| {
                let mut state = make_state(0);
                visit(sender, 2, &mut state);
                state
            },
            |mut state, x| {
                state.count += x.count;
                state.size += x.size;
                state
            },
        );
        println!("{} {} {}", args.file, state.count, state.size);
    }

    Ok(())
}
