//! Disk usage for an ext4 filesystem.

use al_crunch_pool::{execute, Options, Sender};
use al_mmap::Mmap;
use ap_storage::{directory::Iterator, file::File, meta::FileType, Error, FileSystem};
use ap_storage_ext4_ro::{file::Ext4File, Ext4Fs};
use ap_storage_memory::ReadSlice;
use gumdrop::Options as GumdropOptions;
use std::rc::Rc;

#[derive(Debug, GumdropOptions)]
struct CommandOptions {
    /// Display help.
    help: bool,

    /// Direct acccess.
    no_direct: bool,

    /// Leaf optimization
    leaf_optimization: bool,

    /// Number of parallel threads.
    threads: Option<usize>,

    /// Number of slots in the queue per thread.
    #[options(default = "8")]
    slots: usize,

    /// Number of repeats.
    #[options(default = "1")]
    repeat: usize,

    /// Start directory.
    #[options(default = "/")]
    start: String,
}

pub struct WorkerState {
    fs: Rc<Ext4Fs<'static>>,
    size: u64,
    count: usize,
}

fn visit(sender: &Sender<WorkerState>, nr: u64, worker: &mut WorkerState) {
    let fs = Rc::clone(&worker.fs);
    let dir = Ext4File::new(&fs, nr).unwrap();
    let Some(mut iter) = dir.dir() else { return };

    while let Ok(Some(entry)) = iter.next(&mut []) {
        if matches!(entry.typ, FileType::Unknown | FileType::Parent) {
            continue;
        }
        worker.count += 1;

        let Ok(child) = dir.open(entry.offset) else {
            continue;
        };
        worker.size += child.meta().size;

        if entry.typ == FileType::Directory {
            let sender2 = sender.clone();
            sender.send(worker, move |state| {
                visit(&sender2, entry.id, state);
            });
        }
    }
}

fn main() -> Result<(), Error> {
    let opts = CommandOptions::parse_args_default_or_exit();
    let mmap = Mmap::new("/dev/stdin", !opts.no_direct, 0, 0)?;
    let disk = ReadSlice(mmap.0);
    let disk: &(dyn ap_storage::Read + Sync) = &disk;

    let options = Options::default()
        .one_is_zero()
        .io_bound()
        .threads(opts.threads)
        .slots(opts.slots);

    // a function to produce the state for every worker
    let make_state = |_| {
        WorkerState {
            // XXX we don't handle the lifetimes correctly
            fs: Rc::new(
                Ext4Fs::new(unsafe { std::mem::transmute(disk) }, opts.leaf_optimization).unwrap(),
            ),
            size: 0,
            count: 0,
        }
    };

    // the bounded Job queue
    for _i in 0..opts.repeat {
        let state = execute(
            options.clone(),
            make_state,
            |state| (state.count, state.size),
            |sender| {
                let mut state = make_state(0);
                let root = state.fs.root().unwrap();
                let child = root
                    .lookup_path(opts.start.as_bytes())
                    .expect("start directory not found");

                visit(sender, child.meta().id, &mut state);
                (state.count, state.size)
            },
            |mut x, y| {
                x.0 += y.0;
                x.1 += y.1;
                x
            },
        );
        println!("{}\t{}\t{}", opts.start, state.0, state.1);
    }
    Ok(())
}
