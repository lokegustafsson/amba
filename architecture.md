# Project architecture documentation

## Bird's eye view

![Bird's eye view.](https://cdn.discordapp.com/attachments/803687143871742004/1091376154704879696/image.png)

## Introduction

This is an overview of the project's code architecture and aims to give a brief
picture of what resides where and its responsibility rather than a detailed
description of AMBA. 

```md
├── crates
│   ├── amba
│   │   ├── data
│   │   ├── src
│   │   │   ├── init
│   │   │   └── run
│   │   └── templates
│   ├── amba-gui
│   │   └── src
│   ├── bootstrap
│   │   └── src
│   ├── data-structures
│   │   └── src
│   ├── ipc
│   │   └── src
│   ├── libamba
│   │   ├── inc
│   │   └── src
│   ├── libamba-rs
│   │   └── src
│   ├── mitm-debug-stream
│   │   └── src
│   ├── qmp-client
│   │   └── src
│   ├── recipe
│   │   └── src
│   └── s2e-rs
│       ├── cpp_src
│       └── src
... 
```

## How to read this documentation

This documentation is structured such that one can search for a directory's name
and find a corresponding heading with documentation of the relevant components
in the following subheadings. 

## Project structure
AMBA follows a typical project structure where the responsibility of different
components are separated into different directories. All source files reside in
amba/crates/*. 

## `crates/amba`
The amba directory contains the essential parts responsible for connecting and
running all components.

```md
├── amba
│   ├── data
│   ├── src
│   │   ├── init
│   │   └── run
│   └── templates
```

### Creating your own subcommand

cmd.rs specifies how to define a subcommand. A subcommand is a command which is
passed along with `amba`, for example `amba run` is a full command, where `run`
is the subcommand part. There exists currently two subcommands: `init` and `run`
which both define their own execution in the respective directories. 

If one is to implement their own subcommand, apart from having to define what
happens when their subcommand is ran, they also have to define a new entry in
the following enumeration: 

```rust
enum Args {
	Init(InitArgs),
	Run(RunArgs),
	Gui(GuiArgs),
    // new entry here
} 
```

and pass a struct to it, e.g

```rust
pub struct InitArgs {
	#[arg(short, long)]
	force: bool,
	#[arg(short, long)]
	download: bool,
}
```

and then specify what is to happen when it gets pattern matched in main.rs, e.g.

```rust
let res = match args {
    Args::Init(args) => init::init(cmd, data_dir, args)
    // Args::MyCommand(MyArgs) => MyCommand::MyCommand::Run(...)
    ...
    ...
```

### Init subcommand
Init's main purpose is to initialize amba by downloading and building the guest
images that are later ran jointly in S2E, libamba and QEMU.

This is done in several steps:

- init/download.rs tries to download guest images from a given public google drive
    hosted by S2E.
- init/build.rs  after downloading the guest images, build.rs will try to build
    these 
- init/mod.rs is the "runner" part of this module, meaning it wraps the
    subcommand together and is later to be included in amba/src/main.rs

For more technical details, refer to all files in the init/ directory.

### Run subcommand

The idea of this command is to mimic S2E's launch-s2e.sh [script](https://github.com/S2E/s2e-env/blob/master/s2e_env/templates/launch-s2e.sh), meaning it aims to launch QEMU & S2E. The main
difference is that it's written in Rust.

The run subcommand will also create a controller which spawns a number of threads with different tasks:
an ipc thread, a qemu thread and qmp thread. The controller will then loop and
handle various messages, such as `ReplaceBlockGraph` or `ReplaceStateGraph`
which will tell the gui to repaint itself with new graph data.

## crates/bootstrap
This crate mimics the behavior of S2E's bootstrap.sh script. 
This executable will run on startup within the guest, and is responsible
for starting the analyzed executable with a correctly made-symbolic
environment. 

## crates/data-structures
Crate used to modularize our utility data structures. 

## `crates/ipc`
IPC stands for Inter-process communication. This crate contains a structured IPC
implementation utilizing unix sockets to send messages.

## `crates/libamba`
The libamba crate contains the S2E plugin which acts as the driver in amba.
It is through libamba that all data relevant to the analysis is acquired.
Since the gui needs to get updates of the blocks to render a control flow graph,
libamba accomplishes this by hooking a number of modules and listens for changes and sends these to libamba-rs 
where they get processed into a graph

## `crates/mitm-debug-stream`
Debugging tool used to log and print contents of a stream when using the
qmp-client. For more in depth, refer to the source file
mitm-debug-stream/src/lib.rs

Mitm, in this context, is an acronym for Man-in-the-middle.

## `crates/qmp-client`

QMP-client is our own implementation of client communication using the QEMU
Machine Protocol (QMP). The intention with QMP is to communicate directly with
the virtual machine instance that S2E starts. One example of communication could
be querying for current execution state, start and/or stop the virtual machine,
etc. 

## `crates/recipe`
The specification of recipes reside in this directory. 
Recipes purpose is to describe how and what symbolic data is sent to the stdin of the given
binary. Refer to demos/hello.recipe.json for an example. 

A recipe is described using json, but is really a struct:

```rust
pub struct Recipe {
	pub files: BTreeMap<GuestPath, FileSource>,
    // Path to the given binary which is to be analyzed
	pub executable_path: String,
    // filepath to where we're looking for the stdin-file
	pub stdin_path: String,
	pub arg0: Option<String>,
	arguments: Vec<ArgumentSource>,
    // Keeps track of concrete and symbolic variables
	environment: Environment,
}
```

A recipe is later used with the s2ecmd utility to generate symbolic data.

It is convenient out of a user-experience perspective but also necessary to
have a representation of a recipe in any high-level description language as the
data has to pass through FFI (Foreign Function Interface) and later be sent to
the guest in libamba. 

## `crates/s2e-rs`
This crate generates rust code from c++ using autocxx. 
