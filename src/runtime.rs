#![allow(static_mut_refs)]

use std::mem::MaybeUninit;

use gmod::lua;
use tokio::runtime::{Builder, Runtime};
use tokio_util::task::TaskTracker;

pub const DEFAULT_WORKER_THREADS: u16 = 1;
pub const DEFAULT_GRACEFUL_SHUTDOWN_TIMEOUT: u32 = 20;

static mut RUN_TIME: MaybeUninit<Runtime> = MaybeUninit::uninit();
static mut TASK_TRACKER: MaybeUninit<TaskTracker> = MaybeUninit::uninit();
static mut SHUTDOWN_TIMEOUT: u32 = DEFAULT_GRACEFUL_SHUTDOWN_TIMEOUT;

pub(super) fn load(l: lua::State) {
    let worker_threads = get_max_worker_threads(l);
    unsafe {
        SHUTDOWN_TIMEOUT = get_graceful_shutdown_timeout(l);
    }

    let run_time = Builder::new_multi_thread()
        .worker_threads(worker_threads as usize)
        .max_blocking_threads(1)
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    let task_tracker = TaskTracker::new();

    unsafe {
        RUN_TIME = MaybeUninit::new(run_time);
        TASK_TRACKER = MaybeUninit::new(task_tracker);
    }
}

pub(super) fn unload(_: lua::State) {
    let run_time = unsafe { RUN_TIME.assume_init_read() };

    let task_tracker = unsafe { TASK_TRACKER.assume_init_read() };
    task_tracker.close();

    if !task_tracker.is_empty() {
        let timeout = std::time::Duration::from_secs(unsafe { SHUTDOWN_TIMEOUT } as u64);

        run_time.block_on(async {
            tokio::select! {
                _ = task_tracker.wait() => {
                },
                _ = tokio::time::sleep(timeout) => {
                },
            }
        });
    }

    unsafe {
        RUN_TIME = MaybeUninit::uninit();
        TASK_TRACKER = MaybeUninit::uninit();
    }
}

fn read<'a>() -> &'a Runtime {
    unsafe { RUN_TIME.assume_init_ref() }
}

fn read_tracker<'a>() -> &'a TaskTracker {
    unsafe { TASK_TRACKER.assume_init_ref() }
}

pub fn run_async<F>(fut: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    read().spawn(read_tracker().track_future(fut))
}

fn get_max_worker_threads(l: lua::State) -> u16 {
    let mut max_worker_threads = DEFAULT_WORKER_THREADS;

    l.get_global(c"CreateConVar");
    let success = l.pcall_ignore(|| {
        l.push_string("NEO4J_WORKER_THREADS");
        l.push_number(DEFAULT_WORKER_THREADS);
        l.create_table(2, 0);
        {
            l.get_global(c"FCVAR_ARCHIVE");
            l.raw_seti(-2, 1);

            l.get_global(c"FCVAR_PROTECTED");
            l.raw_seti(-2, 2);
        }
        l.push_string("Number of worker threads for the neo4j module");
        1
    });

    if success {
        l.get_field(-1, c"GetInt");
        let success = l.pcall_ignore(|| {
            l.push_value(-2);
            1
        });
        if success {
            max_worker_threads = l.to_number(-1) as u16;
            l.pop();
        }
        l.pop();
    }

    max_worker_threads
}

fn get_graceful_shutdown_timeout(l: lua::State) -> u32 {
    let mut timeout = DEFAULT_GRACEFUL_SHUTDOWN_TIMEOUT;

    l.get_global(c"CreateConVar");
    let success = l.pcall_ignore(|| {
        l.push_string("NEO4J_GRACEFUL_SHUTDOWN_TIMEOUT");
        l.push_number(DEFAULT_WORKER_THREADS);
        l.create_table(2, 0);
        {
            l.get_global(c"FCVAR_ARCHIVE");
            l.raw_seti(-2, 1);

            l.get_global(c"FCVAR_PROTECTED");
            l.raw_seti(-2, 2);
        }
        l.push_string("Timeout for graceful shutdown of the neo4j connections, in seconds");
        1
    });

    if success {
        l.get_field(-1, c"GetInt");
        let success = l.pcall_ignore(|| {
            l.push_value(-2);
            1
        });
        if success {
            timeout = l.to_number(-1) as u32;
            l.pop();
        }
        l.pop();
    }

    timeout
}
