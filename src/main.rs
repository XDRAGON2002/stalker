mod syscalls;

fn main() {
    // Fetch the syscall table, that includes the mappings of syscall ids to their names as well as to their arguments
    // i.e. 0 -> read, 1 -> write ...
    // this is required since we only get the id of the syscall that was triggered not its name
    // so it helps in making the output more human readable
    let syscall_table: std::collections::HashMap<u64, String> = syscalls::fetch_syscall_table();

    // `tracee_pid` will store the pid of the process that we want to talk
    // we can either fork our own process `stalker ls -al` or pass an explicit pid `stalker -p 123`
    // a tracee process needs to exist for the tracer to track
    let mut tracee_pid: u64 = 0; // initializing with garbage value `0`

    // Fetch the commandline arguments this was invoked with
    // and skip the first element since that will be this utility "stalker" itself
    let cmd: Vec<String> = std::env::args()
        .skip(1)
        .collect();

    // Check if the arguments are in `-p 123` format or not
    // i.e. has a specific pid been provided to track
    if cmd.len() == 2 && cmd[0] == "-p" { // If a specific pid has been provided
        tracee_pid = cmd[1].parse::<u64>().unwrap(); // Fetch the pid

        // Trigger a `ptrace` system call on the tracee process to attach it
        // this is required since a tracer can only trace processes it can attach or it already owns
        // since the provided pid could belong to any process, i.e. to any other users, attaching that process is a priviledged operation
        // and only root can do this, this property of `ptrace` is governed by the `YAMA` module on linux,
        // hence the invocation for cases when a pid belongs to a different user needs `sudo`:
        //
        // `$ stalker -p 123` if the pid `123` belongs to your user
        // `$ sudo stalker -p 123` if the pid `123` belongs to some other user
        //
        _ = syscalls::sys_ptrace(16, tracee_pid as i64, 0, 0);
    } else { // If a specific pid was not provided, we presume we want to invoke a new child process with the provided command
        let pid: i64;

        // Trigger a `fork` system call to spin up a new child process for running the provided command
        // store the returned `pid` since that will help us discern between the parent and child
        pid = syscalls::sys_fork();

        // Figure out the opertions for the child and parent
        if pid == 0 { // If we're in the child process
            // Trigger the `ptrace` system call to attach the "TRACE_ME" flag on itself
            // this allows the parent process to trace it without requiring any elevated perms
            _ = syscalls::sys_ptrace(0, 0, 0, 0);

            // Parse the command to be executed as a CString
            // CString ensures that the string is a valid string that would be parsed by a standard C parser
            // It includes things such as ensuring strings end with a termination/null pointer `\0`
            let command: &String = &cmd[0];
            let filename = std::ffi::CString::new(command.as_str()).unwrap();

            // Parse the arguments that need to be passed to the command as well
            let cstr_argv: Vec<std::ffi::CString> = cmd
                .iter()
                .map(|arg| std::ffi::CString::new(arg.as_str()).unwrap())
                .collect();
            
            // Convert the arguments to CStrings as well
            let mut argv: Vec<*const i8> = cstr_argv.iter().map(|s| s.as_ptr()).collect();
            argv.push(std::ptr::null()); // Append `\0` at the end

            // Similar to the arguments, also parse the environment variables
            let cstr_envp: Vec<std::ffi::CString> = std::env::vars()
                .map(|(key, value)| format!("{}={}", key, value))
                .map(|env_str| std::ffi::CString::new(env_str).unwrap())
                .collect();

            // And convert the environment variables to CStrings as well
            let mut envp: Vec<*const i8> = cstr_envp.iter().map(|s| s.as_ptr()).collect();
            envp.push(std::ptr::null()); // Append `\0` at the end

            // Trigger an `execve` systemcall, that will replace the current process call/execution stack
            // with the defined invocation, in our case the invocation is whatever has been passed via the commandline:
            //
            // `$ stalker ls -al /etc/hosts`
            //
            _ = syscalls::sys_execve(filename.as_ptr() as i64, argv.as_ptr() as i64, envp.as_ptr() as i64);

            // `execve` completely replaces the exection stack of the current process
            // if we've reached here, i.e. to this block after we triggered the `execve` system call
            // it's safe to say the `execve` call has failed and we can't trace the process due to this failure
            // so just exit
            eprintln!("failed to exec the requested invocation");
            std::process::exit(1);
        } else if pid < 0 { // The `fork` systemcall returns back a -1 (negative) value in case it fails
            // Since `fork` failed, we don't have a child process to trace
            // so just exit
            eprintln!("failed to fork a new subprocess");
            std::process::exit(1);
        } else { // If we're in the parent process
            // Set the `pid` of the newly spanned child process as the `tracee_pid`
            tracee_pid = pid as u64;
        }
    }

    // We're now out of the `if/else` blocks, at this point:
    // - We definitely have a `tracee process` and the corresponding `tracee_pid` that we want to trace
    //   - If the user provided a pid, we were able to attach it successfully
    //   - If the user provided a command instead, we were able to invoke it in a child process that we own
    // - We're now in the parent process and are now the `tracer process`/`tracer`
    //   - `tracer`, `current`, `original` and `parent` process refer to the same thing, i.e. the process we started off with initially

    // Set a `status`, this is required
    // so it can track the state of the child process
    // we'll invoke a systemcall below that will require this
    let mut status: i64 = 0;

    // Trigger a `wait4` system call for the `tracee`
    // this will make us wait till the state of the `tracee` changes, i.e. it does something
    // at this point what we're essentially waiting for is the `execve` systemcall to be triggered in the `tracee`
    // though it will only be triggered for the child process we spun off and that will be the starting point of the trace
    // if we're tracing an existing process instead i.e. provided via `-p`, then we're just waiting for any action to occur
    // as we'll be tracing the existing process from now on/in the middle and not from the start
    _ = syscalls::sys_wait4(tracee_pid as i64, &mut status as *mut i64 as i64, 0, 0);

    // If we've reached this point, that means the state of the `tracee` has changed
    // and we're ready to start tracing from here on

    // `is_sys_exit` is a toggle variable
    // we will essentially be notified about each system call twice, once during `entry` and again during `exit`
    // this will lead to duplication in our trace, so we use this toggle to ensure we only trace alternative system calls
    // only during exit to be precise since that will also give us the resulting value of the system call
    let mut is_sys_exit: bool = false;

    // This is the main tracing loop, we keep running it until the `tracee` exits
    loop {
        // Trigger a `ptrace` system call, this specific one tells the kernel to inform us when our tracee reaches a system call invocation
        // this included both, entry to the system call as well as exit, this is why we require the `is_sys_exit` toggle flag above
        // this will essentially trap the tracee with a `SIGTRAP` and return control to us
        _ = syscalls::sys_ptrace(24, tracee_pid as i64, 0, 0);

        // Trigger a `wait4` system call, due to the previous `ptrace` system call invocation,
        // we will be notified once the tracee reaches a system call and control will be passed to us, hence we wait till then
        _ = syscalls::sys_wait4(tracee_pid as i64, &mut status as *mut i64 as i64, 0, 0);

        // If we've reached here, that means the `tracee` has reached a system call/or the child has exited

        // Check if the `tracee` has exited or not
        if (status & 0x7f) == 0 { // If it has exited
            // Log the exit status and break out of the loop, i.e. gracefully exit the `tracer` as well since our work is done
            println!("child exited with status: {}", (status >> 8) & 0xff);
            break;
        }

        // If we've reached here, we're sure the `tracee` is alive and has hit a system call
        
        // Check if it's the "exit" invocation from the system call
        if is_sys_exit { // If it is the "exit" invocation
            unsafe {
                // Instantiage a new register state with all zeroes, this is what we'll copy the actual register values of the `tracee` into
                let mut regs: syscalls::UserRegsStruct = std::mem::zeroed();

                // Trigger a `ptrace` system call, this specific instance will read the values from the process registers and copy them
                // in our newly creatd register state so we can access them
                _ = syscalls::sys_ptrace(12, tracee_pid as i64, 0, &mut regs as *mut syscalls::UserRegsStruct as i64);

                // Log the trace for this specific system call which includes the `pid`, `syscall name`, `first few arguments passed in`, `output`:
                //
                // ...
                // [51942] write(1, 55a4553dacf0, 7, ...) = 7
                // [51942] close(1, 55a4553dacf0, 7fae1174a8a0, ...) = 0
                // [51942] close(2, fbad2006, 7fae1174a8a0, ...) = 0
                // ...
                //
                println!(
                    "[{}] {}({:x}, {:x}, {:x}, ...) = {:x}",
                    tracee_pid,
                    syscall_table[&regs.orig_rax],
                    regs.rdi,
                    regs.rsi,
                    regs.rdx,
                    regs.rax,
                );
            }
        }

        // Toggle the flag
        is_sys_exit = !is_sys_exit;
    }
}
