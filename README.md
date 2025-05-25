# Stalker

Stalker is a minimal `system call`/`syscall` tracing utility.

It allows users the ability to trace all the `system calls` being made by processes which is extremely helpful for:
- Debugging
- Figuring out bottlenecks
- Learning about interactions with the Kernel Space
All of this based on lower level primitives and what happens under the hood in the `OS` and `Kernel`.

Inspired from the famous linux utility [strace](https://github.com/strace/strace).

## Key Points

Allows for tracing `system calls` in two ways:
- Provide a command to run and trace it end to end
- Provide the `pid` of an existing process and trace it from that moment

It's also extremely:
- Fast
- Efficient
- Lighweight
- Minimal

This is because:
- It's written in `Rust`
- Uses `Assembly` directly for performing system calls
- Has no dependencies on any external `crate` (apart from `serde` which is being used for some "pretty" output, so can be dropped as it's not important for functioning)

## Concepts Used
- `system calls`
- `ptrace`
- `assembly`
- `rust`
- `registers`

## System Requirements

Since we are using various `system calls` and `linux kernel` specifics, hence a `linux` kernel will be required.

Moreover, since the implementation is extremely specific to `x86_64` architectures, only systems that support those will work.

Environments that will work include:
- `linux` host that runs on `x86_64`
- VM that runs `linux` on `x86_64`

## Setup

- Clone the repo
  ```bash
  $ git clone https://github.com/XDRAGON2002/stalker.git
  $ cd stalker
  ```
- From the root of the repo, run:
  ```bash
  $ cargo build
  ```

## Examples

- From the root of the repo, run:
  ```bash
  $ cargo run -- /usr/bin/ls -al . # Ensure you give the full path of the commands
  ```
- This will run `/usr/bin/ls -al .` as a child process and trace that end to end and produce output:
  ```bash
  ... a lot of subprocess calls ...
  [57362] close(3, 265, 1, ...) = 0
  [57362] fstat(1, 7ffc39331250, 7ffc39331250, ...) = 0
  total 32
  [57362] write(1, 55801bfff4d0, 9, ...) = 9
  [57362] openat(ffffff9c, 7fb90241f929, 80000, ...) = 3
  [57362] fstat(3, 7ffc3932fea0, 7ffc3932fea0, ...) = 0
  [57362] fstat(3, 7ffc3932fcd0, 7ffc3932fcd0, ...) = 0
  [57362] read(3, 55801bfff8e0, 1000, ...) = 12f
  [57362] lseek(3, ffffffffffffff4d, 1, ...) = 7c
  [57362] read(3, 55801bfff8e0, 1000, ...) = b3
  [57362] close(3, 0, 7fb9024528a0, ...) = 0
  drwxrwxr-x 5 dragon dragon 4096 May 25 22:11 .
  [57362] write(1, 55801bfff4d0, 2f, ...) = 2f
  drwxrwxr-x 4 dragon dragon 4096 May 24 23:25 ..
  [57362] write(1, 55801bfff4d0, 30, ...) = 30
  -rw-rw-r-- 1 dragon dragon 2391 May 25 17:35 Cargo.lock
  [57362] write(1, 55801bfff4d0, 38, ...) = 38
  -rw------- 1 dragon dragon  199 May 25 17:35 Cargo.toml
  [57362] write(1, 55801bfff4d0, 38, ...) = 38
  drwxrwxr-x 7 dragon dragon 4096 May 25 22:14 .git
  [57362] write(1, 55801bfff4d0, 32, ...) = 32
  -rw-rw-r-- 1 dragon dragon    8 May 24 23:25 .gitignore
  [57362] write(1, 55801bfff4d0, 38, ...) = 38
  drwxrwxr-x 2 dragon dragon 4096 May 25 20:10 src
  [57362] write(1, 55801bfff4d0, 31, ...) = 31
  drwxrwxr-x 4 dragon dragon 4096 May 25 22:36 target
  [57362] write(1, 55801bfff4d0, 34, ...) = 34
  [57362] close(1, 55801bfff4d0, 7fb9024528a0, ...) = 0
  [57362] close(2, 0, 7fb9024528a0, ...) = 0
  child exited with status: 0
  ```
- To `trace` an exising process, from the root of the repo, run:
  ```bash
  $ sudo cargo run -- -p <pid> # Yes this requires sudo, why? That's explained in the comments of the code
  # For example:
  $ sudo cargo run -- -p 27887
  ```
- This will start tracing the provided process from that moment on:
  ```bash
  $ sudo cargo run -- -p 27887
  [27887] select(1, 7ffd7bd47500, 0, ...) = 1
  [27887] rt_sigprocmask(0, 0, 7f898ca84c20, ...) = 0
  [27887] rt_sigaction(1c, 7ffd7bd47060, 7ffd7bd47100, ...) = 0
  [27887] pselect6(1, 7ffd7bd473a0, 0, ...) = 1
  [27887] read(0, 7ffd7bd4739f, 1, ...) = 1
  [27887] select(1, 7ffd7bd472a0, 0, ...) = 0
  [27887] write(1, e2df0f0, 1, ...) = 1
  [27887] rt_sigaction(1c, 7ffd7bd472b0, 7ffd7bd47350, ...) = 0
  [27887] select(1, 7ffd7bd47500, 0, ...) = 1
  [27887] rt_sigprocmask(0, 0, 7f898ca84c20, ...) = 0
  [27887] rt_sigaction(1c, 7ffd7bd47060, 7ffd7bd47100, ...) = 0
  [27887] pselect6(1, 7ffd7bd473a0, 0, ...) = 1
  [27887] read(0, 7ffd7bd4739f, 1, ...) = 1
  [27887] select(1, 7ffd7bd472a0, 0, ...) = 0
  [27887] write(1, e2df0f0, 1, ...) = 1
  [27887] rt_sigaction(1c, 7ffd7bd472b0, 7ffd7bd47350, ...) = 0
  [27887] select(1, 7ffd7bd47500, 0, ...) = 1
  [27887] rt_sigprocmask(0, 0, 7f898ca84c20, ...) = 0
  [27887] rt_sigaction(1c, 7ffd7bd47060, 7ffd7bd47100, ...) = 0
  [27887] pselect6(1, 7ffd7bd473a0, 0, ...) = 1
  [27887] read(0, 7ffd7bd4739f, 1, ...) = 1
  [27887] select(1, 7ffd7bd472a0, 0, ...) = 0
  [27887] write(1, e2df0f0, 1, ...) = 1
  [27887] rt_sigaction(1c, 7ffd7bd472b0, 7ffd7bd47350, ...) = 0
  
  ```

## Further Improvements
- Extend support to other architectures such as `aarch64`
- Use an actual CLI interface such as `clap` for command and argument parsing and validations
- Make the output easier to read, adding colours could help
- Allow search filtering of `system calls`
- Can also extend this to allow blocking of user defined `system calls`
- Further parse the `syscall` metadata to translate the memory addressed in the outputs to human readable values such as file paths
- Add support for outputing this information as `json` and also outputting things to file
- Add support for summarising system calls

## References and Resources:
- https://blog.rchapman.org/posts/Linux_System_Call_Table_for_x86_64/ (Table defining all `x86_64` system call interfaces)
- https://github.com/JakWai01/lurk (A feature rich implementation of `strace` in `Rust`)
- https://github.com/fasterthanlime/rue (A great resource for how `ptrace` works and a much simpler version of what I built here)
