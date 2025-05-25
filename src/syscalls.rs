#[repr(C)]
#[derive(Debug)]
pub struct UserRegsStruct { // These are all the registers that can be interacted with in the "user space", these ones are specific to `x86_64`, each architecture has it's own set, moreover the system calls also differ from architecture to architecture
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbp: u64,
    pub rbx: u64,
    pub r11: u64,
    pub r10: u64, // fourth argument to the system call is stored here
    pub r9: u64,
    pub r8: u64,
    pub rax: u64, // holds the system call id during "entry" into the system call and holds the result of the system call during "exit"
    pub rcx: u64,
    pub rdx: u64, // third argument to the system call is stored here
    pub rsi: u64, // second argument to the system call is stored here
    pub rdi: u64, // first argument to the system call is stored here
    pub orig_rax: u64, // holds system call id for the system call that was invoked
    pub rip: u64,
    pub cs: u64,
    pub eflags: u64,
    pub rsp: u64,
    pub ss: u64,
    pub fs_base: u64,
    pub gs_base: u64,
    pub ds: u64,
    pub es: u64,
    pub fs: u64,
    pub gs: u64,
}

pub fn fetch_syscall_table() -> std::collections::HashMap<u64, String> {
    // Read the corresponding file which holds the system call mappings and serialize them into JSON
    let syscall_json: serde_json::Value = serde_json::from_str(include_str!("x86_64-syscalls.json"))
        .expect("unable to parse syscalls json");

    // Parse the JSON into key-value pair mappings where each system call id is mapped to it's name
    // i.e. 0 -> read, 1 -> write ...
    let syscall_table: std::collections::HashMap<u64, String> = syscall_json["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            (
                item[0].as_u64().unwrap(),
                item[1].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    
    return syscall_table;
}

pub fn sys_ptrace(rdi: i64, rsi: i64, rdx: i64, r10: i64) -> i64 {
    // Trigger the `ptrace` system call by passing in the correct values to registers in assembly
    // `ptrace` corresponds to system call number `101`, more details can be found via:
    //
    // $ man ptrace
    // $ man 2 ptrace
    //
    let sys_retval: i64;
    unsafe {
        std::arch::asm!(
            "syscall",
            in("rax") 101, // `ptrace` system call id
            in("rdi") rdi, // `request type` for `ptrace`, this modifies the behavior of the call
            in("rsi") rsi, // `tracee pid`
            in("rdx") rdx, // `address` corresponding to the `tracee`
            in("r10") r10, // `data` corresponding to the `tracee`, such as data in `registers`
            lateout("rax") sys_retval // `return value` of the system call
        );
    }

    return sys_retval;
}

pub fn sys_fork() -> i64 {
    // Trigger the `fork` system call by passing in the correct values to registers in assembly
    // `fork` corresponds to system call number `57`, more details can be found via:
    //
    // $ man fork
    // $ man 2 fork
    //
    let sys_retval: i64;
    unsafe {
        std::arch::asm!(
            "syscall",
            in("rax") 57, // `fork` system call id
            lateout("rax") sys_retval // `return value` of the system call
        );
    }

    return sys_retval;
}

pub fn sys_execve(rdi: i64, rsi: i64, rdx: i64) -> i64 {
    // Trigger the `execve` system call by passing in the correct values to registers in assembly
    // `execve` corresponds to system call number `59`, more details can be found via:
    //
    // $ man execve
    // $ man 2 execve
    //
    let sys_retval: i64;
    unsafe {
        std::arch::asm!(
            "syscall",
            in("rax") 59, // `execve` system call id
            in("rdi") rdi, // `filename`/`command` to execute
            in("rsi") rsi, // `arguments` to the `filename`/`command`
            in("rdx") rdx, // `environment variables` to be passed
            lateout("rax") sys_retval // `return value` of the system call
        );
    }

    return sys_retval;
}

pub fn sys_wait4(rdi: i64, rsi: i64, rdx: i64, r10: i64) -> i64 {
    // Trigger the `wait4` system call by passing in the correct values to registers in assembly
    // `wait4` corresponds to system call number `61`, more details can be found via:
    //
    // $ man wait4
    // $ man 2 wait4
    //
    let sys_retval: i64;
    unsafe {
        std::arch::asm!(
            "syscall",
            in("rax") 61, // `wait4` system call id
            in("rdi") rdi, // `pid` to wait on
            in("rsi") rsi, // `status` of the `pid` we're waiting on
            in("rdx") rdx,
            in("r10") r10,
            lateout("rax") sys_retval, // `return value` of the system call
        );
    }

    return sys_retval;
}
