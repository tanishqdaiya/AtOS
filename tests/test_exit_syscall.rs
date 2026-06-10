//! Simulation tests for the sys_exit changes.
//!
//! Recreates the kernel's process table, scheduler, and syscall dispatch
//! in plain std Rust so we can verify the logic without ARM hardware.

// ── Types (mirrors kernel types) ──────────────────────────────────────────

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ProcessState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

#[derive(Debug, Clone, Copy, Default)]
struct ProcessContext {
    x: [u64; 31],
    sp: u64,
    elr: u64,
    spsr: u64,
}

#[derive(Debug, Clone, Copy)]
struct Process {
    pid: u64,
    name: [u8; 32],
    state: ProcessState,
    parent_pid: u64,
    pctx: ProcessContext,
}

impl Process {
    fn new(pid: u64, name: &str, parent_pid: u64, entry_point: u64, sp: u64) -> Self {
        let mut name_bytes = [0u8; 32];
        let bytes = name.as_bytes();
        let len = std::cmp::min(bytes.len(), 32);
        name_bytes[..len].copy_from_slice(&bytes[..len]);
        Self {
            pid,
            name: name_bytes,
            state: ProcessState::Ready,
            parent_pid,
            pctx: ProcessContext { elr: entry_point, sp, ..Default::default() },
        }
    }

    fn set_state(&mut self, new_state: ProcessState) {
        self.state = new_state;
    }
}

// ── ExceptionContext (simplified) ─────────────────────────────────────────

struct ExceptionContext {
    x: [u64; 31],
    elr: u64,
    spsr: u64,
    esr: u64,
    far: u64,
}

// ── Process Table + Scheduler (mirrors kernel logic) ─────────────────────

const MAX_PROCESSES: usize = 50;

static mut PROCESS_TABLE: [Option<Process>; MAX_PROCESSES] = [None; MAX_PROCESSES];
static mut NEXT_PID: u64 = 1;
static mut CURRENT_PROCESS: usize = 0;

/// Mirrors the added `Scheduler::get_current_process_index()`
fn get_current_process_index() -> usize {
    unsafe { CURRENT_PROCESS }
}

fn add_process_to_table(process: Process) -> Result<(), &'static str> {
    unsafe {
        for slot in PROCESS_TABLE.iter_mut() {
            if slot.is_none() {
                *slot = Some(process);
                return Ok(());
            }
        }
    }
    Err("Process table is full")
}

/// Mirrors `choose_next_process()` from scheduler.rs.
///
/// Scheduler behavior:
///  1. If current process is still Running → continue it (return current index)
///  2. Otherwise scan forward circularly from CURRENT_PROCESS+1 for a Ready process
///
/// Because step 2 starts at CURRENT_PROCESS+1, the first `schedule_next()` call
/// (when all processes are Ready) will pick index 1, not 0.
fn choose_next_process() -> Option<usize> {
    unsafe {
        if let Some(current_process) = &PROCESS_TABLE[CURRENT_PROCESS] {
            if current_process.state == ProcessState::Running {
                return Some(CURRENT_PROCESS);
            }
        }
        for i in 1..(MAX_PROCESSES + 1) {
            let idx = (CURRENT_PROCESS + i) % MAX_PROCESSES;
            if let Some(process) = &PROCESS_TABLE[idx] {
                if process.state == ProcessState::Ready {
                    CURRENT_PROCESS = idx;
                    return Some(idx);
                }
            }
        }
    }
    None
}

fn load_pctx(pidx: usize) {
    unsafe {
        if let Some(process) = &mut PROCESS_TABLE[pidx] {
            CURRENT_PROCESS = pidx;
            process.set_state(ProcessState::Running);
        } else {
            panic!("process not found at index {}", pidx);
        }
    }
}

fn schedule_next() -> Option<usize> {
    if let Some(next) = choose_next_process() {
        load_pctx(next);
        Some(next)
    } else {
        None
    }
}

/// Mark current process as Ready (simulates timeslice expiry).
fn timeslice_up() {
    unsafe {
        if let Some(proc) = &mut PROCESS_TABLE[CURRENT_PROCESS] {
            proc.set_state(ProcessState::Ready);
        }
    }
}

// ── Syscall handling (mirrors syscalls.rs) ───────────────────────────────

fn sys_exit(ctx: &mut ExceptionContext) -> Result<(), &'static str> {
    let exit_code = ctx.x[0] as i32;
    eprintln!("[SYS_EXIT] Process exiting with code: {}", exit_code);

    unsafe {
        if let Some(current_process) = &mut PROCESS_TABLE[get_current_process_index()] {
            current_process.set_state(ProcessState::Terminated);
        } else {
            return Err("current process not found");
        }
    }

    let result = schedule_next();
    if result.is_none() {
        eprintln!("[SCHEDULER] No process to schedule!");
    }

    Ok(())
}

fn handle_syscall(ctx: &mut ExceptionContext) -> Result<(), &'static str> {
    let syscall_number: u16 = (ctx.esr & 0xffff) as u16;

    match syscall_number {
        3 => sys_exit(ctx),
        _ => {
            eprintln!("Unknown syscall: {}", ctx.x[8]);
            Ok(())
        }
    }
}

/// Simulates what user-space `exit(exit_code)` does:
/// puts the exit code in x0 and encodes SVC #3 in esr.
///
/// ESR encoding: bits[31:26] = EC=0x15 (SVC), bits[15:0] = imm16 (syscall number)
fn user_exit(exit_code: i32) -> ExceptionContext {
    let mut ctx = ExceptionContext {
        x: [0u64; 31],
        elr: 0,
        spsr: 0,
        esr: (0x15u64 << 26) | 3, // EC=SVC(0x15), imm16=3
        far: 0,
    };
    ctx.x[0] = exit_code as u64;
    ctx
}

/// Simulate svc #1 (print) for completeness
fn user_print(_msg: &str) -> ExceptionContext {
    ExceptionContext {
        x: [0u64; 31],
        elr: 0,
        spsr: 0,
        esr: (0x15u64 << 26) | 1, // EC=SVC(0x15), imm16=1
        far: 0,
    }
}

// ── Reset helper ──────────────────────────────────────────────────────────

fn reset_table() {
    unsafe {
        PROCESS_TABLE = [None; MAX_PROCESSES];
        NEXT_PID = 1;
        CURRENT_PROCESS = 0;
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[test]
fn test_get_current_process_index_initial() {
    reset_table();
    assert_eq!(get_current_process_index(), 0);
}

#[test]
fn test_scheduler_first_schedule_picks_index1_when_two_ready() {
    reset_table();
    // With two Ready processes at [0] and [1], choose_next_process scans
    // from CURRENT_PROCESS+1=1, finds process at index 1 first.
    let p1 = Process::new(1, "proc_a", 0, 0x1000, 0x2000);
    let p2 = Process::new(2, "proc_b", 0, 0x3000, 0x4000);
    add_process_to_table(p1).unwrap();
    add_process_to_table(p2).unwrap();

    schedule_next();
    assert_eq!(get_current_process_index(), 1);
    unsafe {
        assert_eq!(PROCESS_TABLE[1].unwrap().state, ProcessState::Running);
        assert_eq!(PROCESS_TABLE[0].unwrap().state, ProcessState::Ready);
    }
}

#[test]
fn test_scheduler_single_process_wraps_to_index0() {
    reset_table();
    // With only one process at index 0, the scan wraps around and picks it.
    let p = Process::new(1, "solo", 0, 0x1000, 0x2000);
    add_process_to_table(p).unwrap();

    schedule_next();
    assert_eq!(get_current_process_index(), 0);
    unsafe {
        assert_eq!(PROCESS_TABLE[0].unwrap().state, ProcessState::Running);
    }
}

#[test]
fn test_scheduler_continues_running_process() {
    reset_table();
    let p = Process::new(1, "solo", 0, 0x1000, 0x2000);
    add_process_to_table(p).unwrap();

    schedule_next(); // sets Running at index 0
    assert_eq!(get_current_process_index(), 0);

    // Calling schedule_next again — process is Running, so it continues
    schedule_next();
    assert_eq!(get_current_process_index(), 0);
}

#[test]
fn test_timeslice_up_enables_round_robin() {
    reset_table();
    let p1 = Process::new(1, "a", 0, 0x1000, 0x2000);
    let p2 = Process::new(2, "b", 0, 0x3000, 0x4000);
    add_process_to_table(p1).unwrap();
    add_process_to_table(p2).unwrap();

    schedule_next(); // picks index 1 (b Running)
    assert_eq!(get_current_process_index(), 1);

    timeslice_up(); // b → Ready
    schedule_next(); // scans from 2, wraps to 0, picks a
    assert_eq!(get_current_process_index(), 0);
    unsafe {
        assert_eq!(PROCESS_TABLE[0].unwrap().state, ProcessState::Running);
    }
}

#[test]
fn test_sys_exit_sets_process_terminated() {
    reset_table();
    let p = Process::new(1, "test_proc", 0, 0x1000, 0x2000);
    add_process_to_table(p).unwrap();
    schedule_next(); // solo process → index 0 Running

    let mut ctx = user_exit(0);
    sys_exit(&mut ctx).unwrap();

    unsafe {
        assert_eq!(
            PROCESS_TABLE[0].unwrap().state,
            ProcessState::Terminated,
            "process should be Terminated after sys_exit"
        );
    }
}

#[test]
fn test_sys_exit_with_nonzero_code() {
    reset_table();
    let p = Process::new(1, "fail_proc", 0, 0x1000, 0x2000);
    add_process_to_table(p).unwrap();
    schedule_next();

    let mut ctx = user_exit(42);
    sys_exit(&mut ctx).unwrap();

    unsafe {
        assert_eq!(PROCESS_TABLE[0].unwrap().state, ProcessState::Terminated);
    }
}

#[test]
fn test_sys_exit_schedules_next_ready_process() {
    reset_table();
    let p1 = Process::new(1, "proc_a", 0, 0x1000, 0x2000);
    let p2 = Process::new(2, "proc_b", 0, 0x3000, 0x4000);
    add_process_to_table(p1).unwrap();
    add_process_to_table(p2).unwrap();

    schedule_next(); // picks index 1 (b Running)
    assert_eq!(get_current_process_index(), 1);

    // exit b (index 1)
    let mut ctx = user_exit(0);
    sys_exit(&mut ctx).unwrap();

    // After exit, scheduler scans from 2.. wraps to 0 → picks a
    assert_eq!(get_current_process_index(), 0);
    unsafe {
        assert_eq!(PROCESS_TABLE[0].unwrap().state, ProcessState::Running);
    }
}

#[test]
fn test_sys_exit_last_process() {
    reset_table();
    let p = Process::new(1, "only_proc", 0, 0x1000, 0x2000);
    add_process_to_table(p).unwrap();
    schedule_next();

    let mut ctx = user_exit(0);
    sys_exit(&mut ctx).unwrap();

    assert_eq!(choose_next_process(), None);
}

#[test]
fn test_handle_syscall_dispatches_exit() {
    reset_table();
    let p = Process::new(1, "proc", 0, 0x1000, 0x2000);
    add_process_to_table(p).unwrap();
    schedule_next();

    let mut ctx = user_exit(7);
    assert_eq!(ctx.esr & 0xffff, 3); // verify SVC #3 in ESR

    handle_syscall(&mut ctx).unwrap();

    unsafe {
        assert_eq!(PROCESS_TABLE[0].unwrap().state, ProcessState::Terminated);
    }
}

#[test]
fn test_handle_syscall_unknown_number_does_not_panic() {
    reset_table();
    let mut ctx = ExceptionContext {
        x: [0u64; 31],
        elr: 0,
        spsr: 0,
        esr: (0x15u64 << 26) | 0x99, // SVC with unknown number 0x99
        far: 0,
    };
    let result = handle_syscall(&mut ctx);
    assert!(result.is_ok());
}

#[test]
fn test_esr_svc_encoding() {
    // ESR encoding: bits[31:26] = EC, bits[15:0] = imm16 (SVC number)
    let esr_svc3: u64 = (0x15u64 << 26) | 3;
    assert_eq!((esr_svc3 & 0xffff) as u16, 3, "SVC number should be 3");
    assert_eq!((esr_svc3 >> 26) & 0x3f, 0x15, "exception class for SVC");

    let esr_svc1: u64 = (0x15u64 << 26) | 1;
    assert_eq!((esr_svc1 & 0xffff) as u16, 1, "SVC number should be 1");
    assert_eq!((esr_svc1 >> 26) & 0x3f, 0x15);
}

#[test]
fn test_process_new_defaults_to_ready() {
    let p = Process::new(1, "test", 0, 0x1000, 0x2000);
    assert_eq!(p.state, ProcessState::Ready);
    assert_eq!(p.pid, 1);
}

#[test]
fn test_process_set_state() {
    let mut p = Process::new(1, "test", 0, 0x1000, 0x2000);
    assert_eq!(p.state, ProcessState::Ready);
    p.set_state(ProcessState::Running);
    assert_eq!(p.state, ProcessState::Running);
    p.set_state(ProcessState::Terminated);
    assert_eq!(p.state, ProcessState::Terminated);
}

#[test]
fn test_process_table_full() {
    reset_table();
    for i in 0..MAX_PROCESSES {
        let p = Process::new((i + 1) as u64, "fill", 0, 0, 0);
        add_process_to_table(p).unwrap();
    }
    let extra = Process::new(99, "overflow", 0, 0, 0);
    assert_eq!(
        add_process_to_table(extra),
        Err("Process table is full")
    );
}

#[test]
fn test_user_exit_esr_encoding() {
    let ctx = user_exit(5);
    assert_eq!(ctx.x[0], 5, "exit code should be in x0");
    assert_eq!(ctx.esr & 0xffff, 3, "SVC #3");
    assert_eq!((ctx.esr >> 26) & 0x3f, 0x15, "EC should be SVC");
}

#[test]
fn test_multiple_exits_circular() {
    reset_table();
    let p1 = Process::new(1, "a", 0, 0x1000, 0x2000);
    let p2 = Process::new(2, "b", 0, 0x3000, 0x4000);
    let p3 = Process::new(3, "c", 0, 0x5000, 0x6000);
    add_process_to_table(p1).unwrap();
    add_process_to_table(p2).unwrap();
    add_process_to_table(p3).unwrap();

    schedule_next(); // scans from 1, picks b (index 1)
    assert_eq!(get_current_process_index(), 1);

    // exit b → scans from 2, picks c (index 2)
    let mut ctx = user_exit(0);
    sys_exit(&mut ctx).unwrap();
    assert_eq!(get_current_process_index(), 2);

    // exit c → scans from 3.. wraps to 0, picks a (index 0)
    let mut ctx = user_exit(0);
    sys_exit(&mut ctx).unwrap();
    assert_eq!(get_current_process_index(), 0);

    // exit a → no more ready
    let mut ctx = user_exit(0);
    sys_exit(&mut ctx).unwrap();
    assert_eq!(choose_next_process(), None);
}

#[test]
fn test_exit_then_readd_process() {
    reset_table();
    let p1 = Process::new(1, "first", 0, 0x1000, 0x2000);
    add_process_to_table(p1).unwrap();
    schedule_next(); // solo → index 0 Running

    // exit the only process
    let mut ctx = user_exit(0);
    sys_exit(&mut ctx).unwrap();
    assert_eq!(choose_next_process(), None);

    // add a new process after exit
    let p2 = Process::new(2, "second", 0, 0x5000, 0x6000);
    add_process_to_table(p2).unwrap();

    let next = choose_next_process();
    assert!(next.is_some(), "scheduler should find newly added process");
    load_pctx(next.unwrap());
    assert_eq!(get_current_process_index(), 1);
}

#[test]
fn test_exit_code_passed_through_esr() {
    reset_table();
    let p = Process::new(1, "p", 0, 0x1000, 0x2000);
    add_process_to_table(p).unwrap();
    schedule_next();

    for code in [0, 1, -1, 42, -128, 255] {
        reset_table();
        let p = Process::new(1, "p", 0, 0x1000, 0x2000);
        add_process_to_table(p).unwrap();
        schedule_next();

        let mut ctx = user_exit(code);
        assert_eq!(ctx.x[0], code as u64);
    }
}

#[test]
fn test_round_robin_full_cycle() {
    reset_table();
    let p1 = Process::new(1, "a", 0, 0x1000, 0x2000);
    let p2 = Process::new(2, "b", 0, 0x3000, 0x4000);
    add_process_to_table(p1).unwrap();
    add_process_to_table(p2).unwrap();

    // a b both Ready → first schedule picks b (index 1)
    schedule_next();
    assert_eq!(get_current_process_index(), 1);

    // timeslice_up: b → Ready, next picks a (index 0)
    timeslice_up();
    schedule_next();
    assert_eq!(get_current_process_index(), 0);

    // timeslice_up: a → Ready, next picks b (index 1)
    timeslice_up();
    schedule_next();
    assert_eq!(get_current_process_index(), 1);
}
