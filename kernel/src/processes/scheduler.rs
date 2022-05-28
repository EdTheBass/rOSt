use core::any::Any;
use core::cell::RefCell;
use core::intrinsics::type_id;

use alloc::{rc::Rc, vec::Vec};
use utils::get_current_tick;

use super::{Process, Thread};
use crate::processes::dispatcher::run_thread;
use crate::processes::thread::State;

static mut SCHEDULER: Scheduler = Scheduler::new();

pub(crate) fn get_scheduler() -> &'static mut Scheduler {
    unsafe { &mut SCHEDULER }
}

/// Runs the scheduler, giving it control of the CPU.
///
/// Will return only if there are no threads at all to run.
pub fn run_processes() -> Option<()> {
    run_thread(get_scheduler().schedule()?);
}

pub(crate) fn add_thread(thread: Rc<RefCell<Thread>>) {
    get_scheduler().add_thread(thread);
}

pub fn add_process(process: Process) -> Rc<RefCell<Process>> {
    get_scheduler().add_process(process)
}

pub(crate) struct Scheduler {
    /// The currently running process.
    pub running_thread: Option<Rc<RefCell<Thread>>>,
    /// The list of viable threads that are ready to run.
    viable_threads: Vec<Rc<RefCell<Thread>>>,
    /// The list of unviable threads that are not ready to run.
    unviable_threads: Vec<Rc<RefCell<Thread>>>,
    /// The list of processes that are registered.
    processes: Vec<Rc<RefCell<Process>>>,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            running_thread: None,
            viable_threads: Vec::new(),
            unviable_threads: Vec::new(),
            processes: Vec::new(),
        }
    }

    /// Adds a process to the scheduling queue so it will be ran.
    pub fn add_process(&mut self, process: Process) -> Rc<RefCell<Process>> {
        process
            .threads
            .iter()
            .for_each(|thread| self.add_thread(thread.clone()));
        let rc = Rc::new(RefCell::new(process));
        self.processes.push(rc.clone());
        rc
    }

    /// Adds the thread to the queue so it can be ran later.
    pub fn add_thread(&mut self, thread: Rc<RefCell<Thread>>) {.
        let mut _thread = { thread.borrow_mut().state };
        match _thread {
            State::Running => self.viable_threads.push(thread),
            State::NotStarted => self.unviable_threads.push(thread),
            State::Sleeping(time) => {
                self.unviable_threads.push(thread.clone());

                let mut last_real_tick: u64 = 0;

                let __thread =  { thread.borrow_mut() };

                if __thread.last_tick == 0 {
                    last_real_tick = __thread.start_tick;
                } else {
                    last_real_tick = __thread.last_tick;
                }

                if get_current_tick() - last_real_tick > time {
                    _thread = State::Running;
                }
            }
        }
    }

    /// Returns the thread that should be ran next.
    ///
    /// This action removes the thread from the waiting queue - be sure to add it back using `return_thread` if it should be ran again.
    pub fn schedule(&mut self) -> Option<Rc<RefCell<Thread>>> {
        if self.viable_threads.len() == 0 {
            return None;
        }
        Some(self.viable_threads.remove(0))
    }
}
