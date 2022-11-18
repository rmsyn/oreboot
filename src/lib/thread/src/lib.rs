/* SPDX-License-Identifier: GPL-2.0-only */
#![no_std]

use bootstate::{BootState, BootStateSequence};
use smp::boot_cpu;
use util::{cb_err::CbErr, timer::timers_run};
use spin::rwlock::RwLock;

// TODO: add config for x86/non-x86 default
pub const CONFIG_STACK_SIZE: usize = 0x2000;
// TODO: add feature for COOP
pub const CONFIG_NUM_THREADS: usize = 0x4;
pub const NUM_STACK_THREADS: usize = CONFIG_STACK_SIZE * CONFIG_NUM_THREADS;
/// There needs to be at least one thread to run the ramstate state machine.
pub const TOTAL_NUM_THREADS: usize = CONFIG_NUM_THREADS + 1;

static THREAD_STACKS: RwLock<[u8; NUM_STACK_THREADS]> = RwLock::new([0u8; NUM_STACK_THREADS]);
static INITIALIZED: RwLock<bool> = RwLock::new(false);

static ALL_THREADS: RwLock<[Thread; TOTAL_NUM_THREADS]> = RwLock::new([Thread::new(); TOTAL_NUM_THREADS]);

pub struct ThreadMutex {
    locked: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum ThreadState {
    Uninitialized,
    Started,
    Done,
}

pub trait ThreadArg: Sync {}

/// All runnable (but not running) and free threads are kept on their
/// respective lists.
static RUNNABLE_THREADS: RwLock<[Option<&Thread>; TOTAL_NUM_THREADS]> = RwLock::new([None; TOTAL_NUM_THREADS]);
static FREE_THREADS: RwLock<[Option<&Thread>; TOTAL_NUM_THREADS]> = RwLock::new([None; TOTAL_NUM_THREADS]);

static ACTIVE_THREAD: RwLock<Option<Thread>> = RwLock::new(None);

#[derive(Clone, Copy)]
pub struct Thread {
    id: i32,
    stack_current: u64,
    stack_orig: u64,
    next: Option<&'static Thread>,
    entry: Option<fn(&'static dyn ThreadArg) -> Result<(), CbErr>>,
    entry_arg: Option<&'static dyn ThreadArg>,
    can_yield: i32,
    handle: ThreadHandle,
}

impl Thread {
    pub const fn new() -> Self {
        Self {
            id: 0,
            stack_current: 0,
            stack_orig: 0,
            next: None,
            entry: None,
            entry_arg: None,
            can_yield: 0,
            handle: ThreadHandle::new(),
        }
    }

    pub fn can_yield(&self) -> bool {
        self.can_yield > 0
    }

    pub fn set_current_thread(self) {
        assert!(boot_cpu());
        (*ACTIVE_THREAD.write()) = Some(self);
    }

    pub fn schedule(mut self) {
        let c = current_thread();
        self.handle.state = ThreadState::Started; 
        let self_stack = self.stack_current;
        let mut current_stack = if let Some(s) = c { s.stack_current } else { 0 };
        self.set_current_thread();
        switch_to_thread(self_stack, &mut current_stack);
    }
}

pub fn current_thread() -> Option<Thread> {
    *ACTIVE_THREAD.write()
}

pub fn thread_list_empty(list: &[Option<&Thread>]) -> bool {
    let mut ret = true;
    for t in list {
        if t.is_some() {
            ret = false;
            break;
        }
    }
    ret
}

pub fn pop_thread(list: &mut [Option<&'static Thread>]) -> Option<&'static Thread> {
    let t = list[0];
    list[0] = if let Some(a) = t { a.next } else { None };
    t
}

pub fn push_thread(list: &mut [Option<&'static Thread>], thread: &'static Thread) {
    for t in list {
        if t.is_none() {
            *t = Some(thread);
            break;
        }
    }
}

pub fn push_runnable(thread: &'static Thread) {
    push_thread(&mut (*RUNNABLE_THREADS.write()), thread);
}

pub fn pop_runnable() -> Option<&'static Thread> {
    pop_thread(&mut (*RUNNABLE_THREADS.write()))
}

pub fn get_free_thread() -> Option<&'static Thread> {
    if thread_list_empty(&(*FREE_THREADS.write())) {
        None
    } else {
        let t = pop_thread(&mut (*FREE_THREADS.write()));
        if let Some(a) = t {
            if a.stack_orig == 0 {
                return None;
            }
            //a.stack_current = a.stack_orig;
        }
        t
    }
}

pub fn free_thread(thread: &'static Thread) {
    push_thread(&mut (*FREE_THREADS.write()), thread);
}

/// The idle thread is ran whenever there isn't anything else that is runnable.
/// It's sole responsibility is to ensure progress is made by running the timer
/// callbacks.
pub fn idle_thread() {
	/* This thread never voluntarily yields. */
    thread_coop_disable();
    loop {
        timers_run();
    }
}

pub fn thread_coop_enable() {
    if let Some(mut c) = current_thread() {
        assert!(c.can_yield <= 0);
        c.can_yield += 1;
    }
}

pub fn thread_coop_disable() {
    if let Some(mut c) = current_thread() {
        c.can_yield -= 1;
    }
}

#[derive(Clone, Copy)]
pub struct ThreadHandle {
    state: ThreadState,
    error: CbErr,
}

impl ThreadHandle {
    pub const fn new() -> Self {
        Self {
            state: ThreadState::Uninitialized,
            error: CbErr::Err,
        }
    }

    /// Run func(arg) on a new thread. Return () on successful start of thread, < 0
    /// when thread could not be started. The thread handle if populated, will
    /// reflect the state and return code of the thread.
    pub fn run(&mut self, func: fn(& dyn ThreadArg) -> Result<(), CbErr>, arg: & dyn ThreadArg) -> Result<(), CbErr> {
        Err(CbErr::ErrNotImplemented)
    }

    /// thread_run_until is the same as thread_run() except that it blocks state
    /// transitions from occurring in the (state, seq) pair of the boot state
    /// machine.
    pub fn run_until(&mut self, func: fn(& dyn ThreadArg) -> Result<(), CbErr>, arg: & dyn ThreadArg, state: BootState, seq: BootStateSequence) -> Result<(), CbErr> {
        Err(CbErr::ErrNotImplemented)
    }
}
