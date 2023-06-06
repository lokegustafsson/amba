# Meeting Thursday 2023-03-16

## Current project state
1. We are setting up a GUI prototype.
2. Setting up communication between GUI (the amba crate) our S2E plugin
   (libamba) using QMP.
3. Build a control flow graph by our S2E plugin (libamba).

## DONES

- Albin: Got a compiling GUI setup, looked at example egui (GUI framework
  we want to use). Tried to build the GUI with our nix setup. Working on his
  exam.
- Clara: Blocked by Albins PR. Working on her exam.
- Loke: Write a "disjoint set" datastructure needed for graph algorithms Samuel
  is writing for the program graph.
- Loke: Tried run a "hello world" smoke test on our CI server but QEMU crashes
  for not yet known reason.
- Loke: Added functionality to SIGSTOP QEMU. This allows attaching a
  profiler/debugger to QEMU (we want to use GDB for debugging purposes).
- Loke: Documented building the report in the README.
- Samuel: Continued work on control flow graph. Running an analysis takes quite
  a long time (6s without doing anything inside libamba)! We can reduce the
  control flow graph but takes ~800s to only reduce the control flow graph.
  Currently we can produce a control flow graph based on translation of basic
  blocks and try to reduce the graph.
- Samuel: Disabled untested libamba code by removing all calls to it.
- Samuel: Merged linkable and callable libamba-rs.
- Linus: Prepared a pull-request of "ipc-task" and closed because not needed
  anymore.
- Linus: Transfer todos and dones into weeklylog.csv.
- Enaya: Got help from Samuel with the `amba run` issue listed above. Merged
  the fix.
- Enaya: Tried Investigating how to re-run S2E with a modified input so that
  interaction. Noticed we might require QMP to talk to QEMU and currently we
  can't talk to QEMU. Set up so amba starts in QEMU as a server on a thread and
  connect QMP client to it and connect and request some info from QEMU (PR 42).

## TODO
- Albin/Clara: Setup a hello world example GUI and merge to master.
- Albin/Clara: Set up the event handler for GUI so we have a correct GUI
  "server" to work with.
- Loke: Fix impure-run for faster iteration (nix run inside a nix shell works though)
- Samuel: Merge control flow graph skeleton structure. So that we can produce
  todos from that and try to parallelize the work on control flow if possible.
- Loke: Throw symbolic data in amba run. Take `demos/control-flow.c` from
  14 and make the program symbolic over argc (maybe just over getchar()).
  Change make file to run `nix run . -- ...` to run the program in QEMU
  symbolicly.
- Loke: generalize above task (so we can easily make some the program input
  symbolic).
- Loke: Fix QMP branch when it comes to code style and merge it in.
- Samuel: Continue on graph reduction and optimization. Hook on `onStateFork`
  instead of `onTranslateBlockStart` to create symbolic program tree,
  this should be a MUCH smaller graph.
- Enaya/Linux: Investigate what you can do with QMP.
- Enaya: Write about existing tools which is not complete yet in the report
  (Binary ninja, CGC binaries)
- Linus: Create a directory in our git repo containing links to
  related-work-papers. 
- Linus: Summarize a related-work-paper in the report.

## Discussion
- Can we try to merge parts of Samuels PR "control flow 2" and try to
  parallelize more of the work because everything revolves currently around
  that PR.
- We should try to write in the report. But is it worth focusing on it just
  yet? Is our code too unstable?

# Meeting Monday 2023-03-20

## General
- Maybe don't spend more time on it but we need "categories" column for tasks,
  how they are related and how to get a high level view.

## Current project state
1. We are setting up a GUI prototype.
2. Setting up communication between GUI (the amba crate) our S2E plugin
   (libamba) using IPC.
3. Basic control flow graph is done, continued development in reducing the
   graph data being generated, continued development in optimizing the graph
   code (because the complete control flow graph seems to be huge for just a
   hello world program).

Since last week, noticed QMP is not what we should use becuase it is not
mandatory (we don't need it's features) and QMP as a general protocol is
limited and not nice to work with for us. (because we want to send over other
things than like just json)

No GUI updates yet.

## Availability
Iulia: Mondays and Tuesdays (friday every other week) after 15 is fine but not
much later.

|        | Monday               | Tuesday | Wednesday | Thursday | Friday               |
|--------|----------------------|---------|-----------|----------|----------------------|
| Iulia  | -15                  | -15     | ❌        | ❌       | every other week -15 |
| Enaya  | 15-                  | -12     | ✔️         | 15-      | ✔️                    |
| Loke   | every other week 15- | 13-     | ✔️         | ✔️        | ❌                   |
| Samuel | 15-                  | 8-10    | ❌        | 13-      | 8-10, 13-            |
| Clara  | 13-                  | ✔️       | ✔️         | ✔️        | ✔️                    |
| Albin  | 9-                   | ✔️       | ✔️         | ✔️        | ✔️                    |
| Linus  | ✔️                    | ✔️       | ✔️         | 13-      | ✔️                    |
|--------|----------------------|---------|-----------|----------|----------------------|
|        | 15-                  |         |           | 15-      |                      |
|--------|----------------------|---------|-----------|----------|----------------------|

Upcoming meetings preliminary
| Meeting scheduels |                      |                               |
|-------------------|----------------------|-------------------------------|
| LV1               | Monday 15-           | Thursday 13                   |
| LV2               | Tuesday 10-          | Thursday 13                   |
| Easter            | Monday 10- (distans) | Thursday 13- (distans)        |
| LV3               | Wednesday 15-        | Friday 15-                    |
| LV4               | Tuesday 10-          | Friday 13-                    |
| LV5               | Monday 15-           | Thursday 13- (report meeting) |
| LV6               | Tueday 10-           | Friday 13-                    |
| LV7               | Tuesday 10-          | Friday 13-                    |
| LV8               | Tuesday 9.30-        | Friday 13-                    |
| LV9               | every day 6-18       |                               |


## Iulia report comments
- Chapters on highest level
- Missing (symbolic execution, avgränsningar, subsection)
- Avgränsningar - mixing apples and oranges.
- Talk about symbolic execution - go into more details. Give an intuition. Take
  inspiration of depth level from related works.

## DONES
- Loke: Setup IPC instead of QMP, which is useful for us amba ↔ libamba. Data
  is serialized rust structs using serde bincode. Not merged because untested.
- Clara: Written the hello-world gui prototype. Yet to merge to master.
- Samuel: Merge control flow graph skeleton structure. So that we can produce
  todos from that and try to parallelize the work on control flow if possible.
- Loke: Fix QMP branch when it comes to code style and merge it in.
- Enaya/Linux: Investigate what you can do with QMP. We don't want or need
  QMP... QMP is not needed because we don't need to touch low level QEMU stuff
  and we don't want to use it as general protocol because it is limited to
  json.
- Loke: Fix impure-run for faster iteration (nix run inside a nix shell works though)
- Loke: Found solution to hello-world CI crashing on his CI-server issue and
  fixed it. Some options that are not supported on that hardware...
- Linus: Create a directory in our git repo containing links to
  related-work-papers. Discontinued because easier to put it in the citations
  list instead.
- Enaya: Translate an addr to symbol+offset in a binary using
  dwarf(debug data format) debug data. Discontinued because we don't really
  need this and it is an extra thing. If debug info doesn't exist we won't have
  any symbol data... A bit early to focus on things like this.
- Linus: Summarize a related-work-paper in the report. Not yet pushed.
- Linus: Adding fancy coverpage in report to meet the report requirements.
- Albin/Clara: Set up the event handler for GUI running in the main thread.

## In-progress
- Albin: build the our GUI code using nix. In-progress.
- Loke: Throw symbolic data in amba run. Take `demos/control-flow.c` from
  14 and make the program symbolic over argc (maybe just over getchar()).
  Change make file to run `nix run . -- ...` to run the program in QEMU
  symbolicly. In-progress.
- Loke: generalize above task (so we can easily make some the program input
  symbolic). In-progress.
- Samuel: Hook on `onStateFork` instead of `onTranslateBlockStart` to create
  symbolic program tree, this should be a MUCH smaller graph. In-progress.

## TODOS (also refer to In-progress):
- Enaya: Integrate amba ↔ libamba by sending our graph data to GUI and display
  it. Start with some small data that you send over.
- Samuel: Iterative graph building.
- Linus: Visualize a few graph nodes with edges with some text inside. The text
  should be updatable. We want to use this to visualize the control-flow graph.
- Loke: Set up a way to run both GUI and amba.

- Enaya: Report: Use a sensible documentclass (report), use \chapter for top
  level sections.
- Clara: Report: section about symbolic execution, go into more details. Give an
  intuition. Take inspiration of depth level from related works.
- Loke: Report: draft of sections we want in the report.
- Albin: Rebase out all downloaded pdf's from the repo (because we don't want
  pdf's that might have copyright issues in our repo. We would like to make the
  repo public eventually.)
- Clara: Setup a functional virtual machine to be able to run the GUI.

## Extra Ideas
- Let's have dedicated breaks during meetings, 45m - pause - 45m
- Maybe go through normal meeting stuffs (done, todos) without losing focus and
  have discussions + drama + coding help etc. afterwards.

# Meeting Thursday 2023-03-23

## Current project state
- We have a clean repo now free from copyrighted pdfs.
- We have a GUI we can start developing.
- We have IPC that we can create connections with and communicate between GUI
  and our S2E plugin.
- We can create a CFG of a binary, the "full" and the symbolic CFG. We want to
  start visualizing this.

## DONES
- Albin: build the our GUI code using nix.
- Loke: Throw symbolic data in amba run. Take `demos/control-flow.c` from
  14 and make the program symbolic over argc (maybe just over getchar()).
  Change make file to run `nix run . -- ...` to run the program in QEMU
  symbolicly.
- Loke: generalize above task (so we can easily make some the program input
  symbolic). Done, using "Recipe" files.
- Samuel: Iterative graph building.
- Albin: Rebase out all downloaded pdf's from the repo (because we don't want
  pdf's that might have copyright issues in our repo. We would like to make the
  repo public eventually.)
- ~~Linus~~: Visualize a few graph nodes with edges with some text inside. The text
  should be updatable. We want to use this to visualize the control-flow graph.
  Done by hello-world gui task.

## In-progress
- Loke: ~~Enaya~~: Integrate amba ↔ libamba by sending our graph data to GUI
  and display it. Start with some small data that you send over. No-progress
  yet. Firstly cleanup GUI side first so people can start doing stuff there
  without being blocked.
- Enaya: Report: Use a sensible documentclass (report), use \chapter for top
  level sections. No-progress yet.
- Clara: Report: section about symbolic execution, go into more details. Give
  an intuition. Take inspiration of depth level from related works.
- Linus: ~~Loke~~: Report: draft of sections we want in the report.
- Clara: Setup a functional virtual machine to be able to run the GUI.
- Enaya: Look at existing S2E plugins trying to figure out how to map virtual
  addresses to native ones. So we can refer to a disassembler and understand
  where in the code we are when running in QEMU. TODO: include the computed
  native addr in CFG node as meta data.
- Enaya: Reduce the CFG to only include blocks that are inside our "module" and
  not OS modules and etc. TODO: Get module path in the plugin via lua config
  file.
- Samuel: Hook on `onStateFork` instead of `onTranslateBlockStart` to create
  symbolic program tree, this should be a MUCH smaller graph. In-progress.

## TODOS (also refer to In-progress)
- Samuel: Factor out data structures to own crate. (SmallU64Set, DisjointSet,
  Graph)
- Samuel: Create a metadata framework for the CFG nodes.
- Albin: Use capstone-rs to disassemble a slice of bytes. Later on this
  will be integrated within the GUI disassembly view. 
- UNASSIGNED: Send over instruction slice for a node when it is new. Send this
  only every so often.
- UNASSIGNED: Use addr2line to get as much debug info out as possible from the
  binary on disk. Use Enayas tmp code and fix it. We want something like source
  code line corresponding to a chunk of assembly as a comment.
- UNASSIGNED: ~~Loke~~: Debug libamba code. Libamba (bootleg valgrind
  reimplementation) is currently broken and needs to be debugged to figure out
  what's wrong.
- Enaya: REPORT (URGENT) Send in revised report to library guidance.
- Enaya: REPORT (URGENT) Fix current citations by me to be more relevant and
  academical.
- Samuel: REPORT Describe CFG compression a little bit in the graph, why and
  how.
- Samuel: Detect and identify strongly-connected components, this can be a
  higher level view of the CFG.
- Linus: REPORT Read through whole report and create a plan to start
  attacking.
- Clara: Represent compressed nodes as an ordered list of original nodes. This
  will help in the implementation of compression algorithm and also be needed
  to reconstruct the disassembly of compressed nodes on the GUI side.
- UNASSIGNED: create a subcommand that creates a recipe json with no symbolic
  input.
- UNASSIGNED: create an example recipe file with all features displayed and
  commented (documented).
- Linus: Write a project arcitecture overview, so that everyone has an
  idea of what every part of the project does.


## Discussions
- Loke: One can try to do some coloring using strongly-connected components in
  the CFG.
- Loke: One can try to do automatic state-merging. Try guessing a state-merging
  strategy from the state-fork graph and see if it improves the CFG, revert
  otherwise?
- Maybe have a wizard for creating recipe files, for the user-friendliness.

# Möte 2023-03-28

## Bibliotekshandledning
Rimliga feedback.

## DONES
- Clara: Report: section about symbolic execution, go into more details. Give
  an intuition. Take inspiration of depth level from related works.
- Clara: Setup a functional virtual machine to be able to run the GUI.
- Enaya: Look at existing S2E plugins trying to figure out how to map virtual
  addresses to native ones. So we can refer to a disassembler and understand
  where in the code we are when running in QEMU.
- Enaya: Reduce the CFG to only include blocks that are inside our "module" and
  not OS modules and etc. Get module path in the plugin via lua config file.
- Samuel: Hook on `onStateFork` instead of `onTranslateBlockStart` to create
  symbolic program tree, this should be a MUCH smaller graph.
- Samuel: Factor out data structures to own crate. (SmallU64Set, DisjointSet,
  Graph)
- Enaya: REPORT (URGENT) Send in revised report to library guidance.
- Enaya: REPORT (URGENT) Fix current citations by me to be more relevant and
  academical.

## In-progress
- Loke: ~~Enaya~~: Integrate amba ↔ libamba by sending our graph data to GUI
  and display it. Start with some small data that you send over. No-progress
  yet. Firstly cleanup GUI side first so people can start doing stuff there
  without being blocked. IN-PROGRESS. Bug in our IPC crate, Data is not
  recieved.
- Enaya: Report: Use a sensible documentclass (report), use \chapter for top
  level sections. NO-PROGRESS YET.
- Linus: ~~Loke~~: REPORT draft of sections we want in the report. We need
  something like Introduction - Theory - Method - Results/Discussion.
  IN-PROGRESS. Needs discussion.
- Samuel: Create a metadata framework for the CFG nodes. IN-PROGRESS.
- Albin: Use capstone-rs to disassemble a slice of bytes. Later on this
  will be integrated within the GUI disassembly view. IN-PROGRESS.
- Enaya: ~~UNASSIGNED~~: Use addr2line to get as much debug info out as
  possible from the binary on disk. Use Enayas tmp code and fix it. We want
  something like source code line corresponding to a chunk of assembly as a
  comment. IN-PROGRESS.
- Samuel: Detect and identify strongly-connected components, this can be a
  higher level view of the CFG. IN-PROGRESS. We have two algorithms for this,
  we want a DAG as output but the algorithms don't give that as output.
  Complexity is a problem to do that.
- Linus: REPORT Read through whole report and create a plan to start
  attacking. IN-PROGRESS.
- Clara: Represent compressed nodes as an ordered list of original nodes. This
  will help in the implementation of compression algorithm and also be needed
  to reconstruct the disassembly of compressed nodes on the GUI side. IN-PROGRESS.
- UNASSIGNED: create a subcommand that creates a recipe json with no symbolic
  input. NO-PROGRESS. Maybe we just need an example file one can copy and edit.
- UNASSIGNED: create an example recipe file with all features displayed and
  commented (documented). NO-PROGRESS.
- Linus: Write a project arcitecture overview, so that everyone has an
  idea of what every part of the project does. IN-PROGRESS 65.

## TODOS (also refer to In-progress)
- UNASSIGNED: include the computed native addr in CFG node as metadata.
  NO-PROGRESS. Blocked by the metadata struct implementation.
- UNASSIGNED: Send over instruction slice for a node when it is new. Send this
  only every so often. NO-PROGRESS.
- UNASSIGNED: ~~Loke~~: Debug libamba code. Libamba (bootleg valgrind
  reimplementation) is currently broken and needs to be debugged to figure out
  what's wrong. NO-PROGRESS.
- Samuel: REPORT Describe CFG compression a little bit in the graph, why and
  how. NO-PROGRESS.
- Enaya: Create a IpcGraph having edges and metadata bidirectional
  conversion with existing data_structures::Graph. It makes it performent over
  IPC without making data_structures::Graph more specific, leaves it to be
  General.
- UNASSIGNED: Rewrite recipe crate to do more syntax and semantic checks to
  catch more syntactically correct but semantically incorrect recipe files.
  Example: that the binary is statically linked etc.

# Möte 2023-03-30

## Current project state
- We have an unmerged but pretty complete integration of GUI and amba.
- We need to send right things, fetch right things and an actual pretty Graph
  rendering.

## DONES
- Enaya: Use addr2line to get as much debug info out as
  possible from the binary on disk. Use Enayas tmp code and fix it. We want
  something like source code line corresponding to a chunk of assembly as a
  comment.
- Samuel: Create a metadata framework for the CFG nodes.
- Loke: ~~Enaya~~: Integrate amba ↔ libamba by sending our graph data to GUI
  and display it. Start with some small data that you send over. No-progress
  yet. Firstly cleanup GUI side first so people can start doing stuff there
  without being blocked. IN-PROGRESS. Not merged.
- Loke: ~~Enaya~~: Create a IpcGraph having edges and metadata bidirectional
  conversion with existing data_structures::Graph. It makes it performent over
  IPC without making data_structures::Graph more specific, leaves it to be
  General. Not merged.

## In-progress
- Enaya: REPORT Use a sensible documentclass (report), use \chapter for top
  level sections. NO-PROGRESS YET.
- Linus: ~~Loke~~: REPORT draft of sections we want in the report. We need
  something like Introduction - Theory - Method - Results/Discussion.
  IN-PROGRESS. Needs discussion.
- Albin: Use capstone-rs to disassemble a slice of bytes. Later on this
  will be integrated within the GUI disassembly view. IN-PROGRESS.
- Samuel: Detect and identify strongly-connected components, this can be a
  higher level view of the CFG. IN-PROGRESS. We have two algorithms for this,
  we want a DAG as output but the algorithms don't give that as output.
  Complexity is a problem to do that.
- Linus: REPORT Read through whole report and create a plan to start
  attacking. IN-PROGRESS.
- Clara: Represent compressed nodes as an ordered list of original nodes. This
  will help in the implementation of compression algorithm and also be needed
  to reconstruct the disassembly of compressed nodes on the GUI side. IN-PROGRESS.
- UNASSIGNED: create a subcommand that creates a recipe json with no symbolic
  input. NO-PROGRESS. Maybe we just need an example file one can copy and edit.
- UNASSIGNED: create an example recipe file with all features displayed and
  commented (documented). NO-PROGRESS.
- Linus: Write a project arcitecture overview, so that everyone has an
  idea of what every part of the project does. IN-PROGRESS 65.
- UNASSIGNED: include the computed native addr in CFG node as metadata.
  NO-PROGRESS. Blocked by the metadata struct implementation.
- UNASSIGNED: Send over instruction slice for a node when it is new. Send this
  only every so often. NO-PROGRESS.
- UNASSIGNED: Debug libamba code. Libamba (bootleg valgrind
  reimplementation) is currently broken and needs to be debugged to figure out
  what's wrong. NO-PROGRESS.
- Samuel: REPORT Describe CFG compression a little bit in the graph, why and
  how. NO-PROGRESS.
- UNASSIGNED: Rewrite recipe crate to do more syntax and semantic checks to
  catch more syntactically correct but semantically incorrect recipe files.
  Example: that the binary is statically linked etc.

## TODOS (also refer to In-progress)
- Loke: **GRAPH RENDERING** Iterative and interactive graph layout
  algorithm (changing layout parameters in the gui).
- Loke: **GRAPH RENDERING** Some sort of force field that makes the graph
  "hang top down from the starting node".
- Loke: **GRAPH RENDERING** LOW PRIO Improve time complexity using
  Barnes-Hut/Quadtree, currently ≈ n^3 in number of nodes.
- UNASSIGNED: CGC binaries.
- Enaya: Store KLEE expressions as metadata to get to some node (from the
  starting node). Evaluate it to a printable expression.
- UNASSIGNED: GUI tool to choose to prioritize a certain "state" in S2E. So
  that one can focus on a branch that is intresting and ignore others manually.
  This will give more control to the user. Scheduele a state to run later?
- UNASSIGNED: Monitor syscalls made by the binary (hook on `onSyscall`? Is
  there such a thing?).
- Samuel: Have our own alternative naming for S2E states. Now a fork leads
  to one new state and one of the states is continued in the current state, but
  for rendering a graph we need unique names for the state continued after a
  fork and before.
- UNASSIGNED: Figure out how to do useful state-merging using S2E. Maybe we
  could do something like creating breakpoints for start and end addresses the
  user has chosen or do something automatic if possible, can this be done using
  S2E with some hooks?. Investigation task.

# Möte 2023-04-03

- Albin: worked on his capstone-rs task, hasn't written any code yet.
- Clara: worked on her compressed nodes task.
- Loke: worked on drawing graphs
- Enaya: KlEE expressions
- Linus: Worked on report/architecture docs

## Suggestions for structure in report
- Introduction
  - 1/3 should present the problem in the introduction (continously narrowing
  until we get to the solution)
- Background into its own section
- (AMBA) After background we need a section where we describe what we're doing (feature
    set)
- Implementation section (briefly describing what we have and what decisions
    were taken)
- Evaluation section (comparison with other frameworks or tools)
- Limitations section
- Related work (discussing related work in-depth)
- Conclusion section

## Misc 
- Title suggestion: Developer assisted binary analysis
- Do not use hybrid in the title because it is misleading (implies that we're using static &
    dynamic analysis)

## DONES
- Linus: ~~Loke~~: REPORT draft of sections we want in the report. We need
  something like Introduction - Theory - Method - Results/Discussion.
- Samuel: Detect and identify strongly-connected components, this can be a
  higher level view of the CFG. We have two algorithms for this,
  we want a DAG as output but the algorithms don't give that as output.
  Complexity is a problem to do that. Add test case
- Loke: **GRAPH RENDERING** Iterative and interactive graph layout
  algorithm (changing layout parameters in the gui).
- Loke: **GRAPH RENDERING** Some sort of force field that makes the graph
  "hang top down from the starting node".

## Backlog
- Enaya: REPORT Use a sensible documentclass (report), use \chapter for top
  level sections. NO-PROGRESS YET.
- Albin: Use capstone-rs to disassemble a slice of bytes. Later on this
  will be integrated within the GUI disassembly view. IN-PROGRESS.
- Linus: REPORT Read through whole report and create a plan to start
  attacking. IN-PROGRESS.
- Clara: Represent compressed nodes as an ordered list of original nodes. This
  will help in the implementation of compression algorithm and also be needed
  to reconstruct the disassembly of compressed nodes on the GUI side. IN-PROGRESS.
- UNASSIGNED: create a subcommand that creates a recipe json with no symbolic
  input. NO-PROGRESS. Maybe we just need an example file one can copy and edit.
- UNASSIGNED: create an example recipe file with all features displayed and
  commented (documented). NO-PROGRESS.
- Linus: Write a project arcitecture overview, so that everyone has an
  idea of what every part of the project does. IN-PROGRESS 65.
- UNASSIGNED: include the computed native addr in CFG node as metadata.
  NO-PROGRESS. Blocked by the metadata struct implementation.
- UNASSIGNED: Send over instruction slice for a node when it is new. Send this
  only every so often. NO-PROGRESS.
- UNASSIGNED: Debug libamba code. Libamba (bootleg valgrind
  reimplementation) is currently broken and needs to be debugged to figure out
  what's wrong. NO-PROGRESS.
- UNASSIGNED: Rewrite recipe crate to do more syntax and semantic checks to
  catch more syntactically correct but semantically incorrect recipe files.
  Example: that the binary is statically linked etc.
- UNASSIGNED: CGC binaries. NO PROGRESS
- Enaya: Store KLEE expressions as metadata to get to some node (from the
  starting node). Evaluate it to a printable expression. IN-PROGRESS
- UNASSIGNED: GUI tool to choose to prioritize a certain "state" in S2E. So
  that one can focus on a branch that is intresting and ignore others manually.
  This will give more control to the user. Scheduele a state to run later?
- UNASSIGNED: Monitor syscalls made by the binary (hook on `onSyscall`? Is
  there such a thing?).
- Samuel: Have our own alternative naming for S2E states. Now a fork leads
  to one new state and one of the states is continued in the current state, but
  for rendering a graph we need unique names for the state continued after a
  fork and before. NO-PROGRESS
- UNASSIGNED: Figure out how to do useful state-merging using S2E. Maybe we
  could do something like creating breakpoints for start and end addresses the
  user has chosen or do something automatic if possible, can this be done using
  S2E with some hooks?. Investigation task.
- Loke: **GRAPH RENDERING** LOW PRIO Improve time complexity using
  Barnes-Hut/Quadtree, currently ≈ n^3 in number of nodes.

## TODOS (also refer to backlog)
1. Introduction
  - 1/3 should present the problem in the introduction (continously narrowing
  until we get to the solution)
2. Background into its own section. Should be able to skip reading this section
   if one knows theory in the domain. (example if one knows what symbolic
   execution is)
3. AMBA
  - After background we need a section where we describe what we're doing
	(feature set)
4. Implementation section (briefly describing what we have and what decisions
    were taken)
5. Evaluation section (comparison with other frameworks or tools)
6. Limitations section (what amba is what amba is not, what is does better,
   what are its shortcommings.)
7. Related work (discussing related work in-depth)
8. Conclusion section

Graph compression, strongly connected components, dynamic and static disassembly,
debug data, graph rendering and layouting, symbolic input & state forking, state
merging, state prioritization, analysis setup-wizard

- Clara: write about related work paper (X-force)
- Loke: Feature set section, discuss s2e vs symQEMU vs angr vs symCC 
- Samuel: Describe CFG compression a little bit in the graph, why and
  how. Strongly connected components
- Albin: Introduction (introduce the topic, describe the problem and background
    and our purpose without going too much into detail)
- Linus: Symbolic input & state forking, evaluation section
- Enaya: debug data, limitations section

# Möte 2023-04-06

## DONES
- Samuel+*Loke*: Have our own alternative naming for S2E states. Now a fork leads
  to one new state and one of the states is continued in the current state, but
  for rendering a graph we need unique names for the state continued after a
  fork and before. DONE, at the same time as Loke.

## Backlog
- Enaya: **REPORT** Use a sensible documentclass (report), use \chapter for top
  level sections. NO-PROGRESS YET.
- Enaya: **REPORT** debug data, limitations section. NO-PROGRESS.
- Enaya: Store KLEE expressions as metadata to get to some node (from the
  starting node). Evaluate it to a printable expression. IN-PROGRESS

- Albin: Use capstone-rs to disassemble a slice of bytes. Later on this
  will be integrated within the GUI disassembly view. IN-PROGRESS.
- Albin: **REPORT** Introduction (introduce the topic, describe the problem and background
    and our purpose without going too much into detail)

- Linus: **REPORT** Read through whole report and create a plan to start
  attacking. IN-PROGRESS.
- Linus: Write a project arcitecture overview, so that everyone has an
  idea of what every part of the project does. IN-PROGRESS 65.
- Linus: **REPORT** Symbolic input & state forking, evaluation section

- Clara: Represent compressed nodes as an ordered list of original nodes. This
  will help in the implementation of compression algorithm and also be needed
  to reconstruct the disassembly of compressed nodes on the GUI side. IN-PROGRESS.
- Clara: **REPORT** write about related work paper (X-force)

- Loke: **GRAPH RENDERING** LOW PRIO Improve time complexity using
  Barnes-Hut/Quadtree, currently ≈ n^3 in number of nodes.
- Loke: **REPORT** Feature set section, discuss s2e vs symQEMU vs angr vs symCC 

- Samuel: **REPORT** Describe CFG compression a little bit in the graph, why and
  how. Strongly connected components

- UNASSIGNED: create a subcommand that creates a recipe json with no symbolic
  input. NO-PROGRESS. Maybe we just need an example file one can copy and edit.
- UNASSIGNED: create an example recipe file with all features displayed and
  commented (documented). NO-PROGRESS.
- UNASSIGNED: include the computed native addr in CFG node as metadata.
  NO-PROGRESS. Blocked by the metadata struct implementation.
- UNASSIGNED: Send over instruction slice for a node when it is new. Send this
  only every so often. NO-PROGRESS.
- UNASSIGNED: Debug libamba code. Libamba (bootleg valgrind
  reimplementation) is currently broken and needs to be debugged to figure out
  what's wrong. NO-PROGRESS.
- UNASSIGNED: Rewrite recipe crate to do more syntax and semantic checks to
  catch more syntactically correct but semantically incorrect recipe files.
  Example: that the binary is statically linked etc.
- UNASSIGNED: CGC binaries. NO PROGRESS
- UNASSIGNED: GUI tool to choose to prioritize a certain "state" in S2E. So
  that one can focus on a branch that is intresting and ignore others manually.
  This will give more control to the user. Scheduele a state to run later?
- UNASSIGNED: Monitor syscalls made by the binary (hook on `onSyscall`? Is
  there such a thing?).
- UNASSIGNED: Figure out how to do useful state-merging using S2E. Maybe we
  could do something like creating breakpoints for start and end addresses the
  user has chosen or do something automatic if possible, can this be done using
  S2E with some hooks?. Investigation task.

## TODOS (also refer to backlog)
- Samuel: pull out some stuff from the README.md to DEVELOPMENT.md and add
  `cargo2nix -f` bla bla in there too.
- Samuel: Rework PR 69 to be more in line with existing setup expecially
  regarding the FFI boundary.

# Möte 2023-04-12

## DONES
- Albin~~Enaya~~: **REPORT** Use a sensible documentclass (report), use
  \chapter for top level sections.
- Albin: **REPORT** Introduction (introduce the topic, describe the problem and
  background and our purpose without going too much into detail). NOT MERGED.
- Linus: **REPORT** Read through whole report and create a plan to start
  attacking.
- Linus: Write a project arcitecture overview, so that everyone has an
  idea of what every part of the project does.
- Clara: **REPORT** write about related work paper (X-force)
- Samuel: pull out some stuff from the README.md to DEVELOPMENT.md and add
  `cargo2nix -f` bla bla in there too.

## Backlog
- Enaya: **REPORT** debug data, limitations section. NO-PROGRESS.
- Enaya: Store KLEE expressions as metadata to get to some node (from the
  starting node). Evaluate it to a printable expression. A S2EExecutionState is
  what we need but that is not storable, not sure if we even need to store
  this. One could get a concrete input which can get us to this state which is
  simple but not a complete thing we want. Hard with no good docs for KLEE
  architecture and S2E architecture that goes into detail. IN-PROGRESS

- Albin: Use capstone-rs to disassemble a slice of bytes. Later on this
  will be integrated within the GUI disassembly view. IN-PROGRESS.

- Linus: **REPORT** Symbolic input & state forking, evaluation section.
  IN-PROGRESS.

- Clara: Represent compressed nodes as an ordered list of original nodes. This
  will help in the implementation of compression algorithm and also be needed
  to reconstruct the disassembly of compressed nodes on the GUI side.
  IN-PROGRESS.

- Loke: **GRAPH RENDERING** LOW PRIO Improve time complexity using
  Barnes-Hut/Quadtree, currently ≈ n^3 in number of nodes.
- Loke: **REPORT** Feature set section, discuss s2e vs symQEMU vs angr vs symCC 
- Loke: include the computed native addr in CFG node as metadata.
  NO-PROGRESS. Implemented in 70 which is blocked by 78.

- Samuel: **REPORT** Describe CFG compression a little bit in the graph, why and
  how. Strongly connected components. IN-PROGRESS.
- Samuel: Rework PR 69 to be more in line with existing setup expecially
  regarding the FFI boundary. Split into 77 (reviewable) and 78
  (IN-PROGRESS).

- UNASSIGNED: create a subcommand that creates a recipe json with no symbolic
  input. NO-PROGRESS. Maybe we just need an example file one can copy and edit.
- UNASSIGNED: create an example recipe file with all features displayed and
  commented (documented). NO-PROGRESS.
- UNASSIGNED: Send over instruction slice for a node when it is new. Send this
  only every so often. NO-PROGRESS.
- UNASSIGNED: Debug libamba code. Libamba (bootleg valgrind
  reimplementation) is currently broken and needs to be debugged to figure out
  what's wrong. NO-PROGRESS.
- UNASSIGNED: Rewrite recipe crate to do more syntax and semantic checks to
  catch more syntactically correct but semantically incorrect recipe files.
  Example: that the binary is statically linked etc.
- UNASSIGNED: CGC binaries. NO PROGRESS
- UNASSIGNED: GUI tool to choose to prioritize a certain "state" in S2E. So
  that one can focus on a branch that is intresting and ignore others manually.
  This will give more control to the user. Scheduele a state to run later?
- UNASSIGNED: Monitor syscalls made by the binary (hook on `onSyscall`? Is
  there such a thing?).
- UNASSIGNED: Figure out how to do useful state-merging using S2E. Maybe we
  could do something like creating breakpoints for start and end addresses the
  user has chosen or do something automatic if possible, can this be done using
  S2E with some hooks?. Investigation task.

## TODOS (also refer to backlog)

# Meeting 2023-04-12
Refactored the report structure a whole bunch.

## DONES

## Backlog
- Enaya: **REPORT** debug data, limitations section. NO-PROGRESS.
- Enaya: Store KLEE expressions as metadata to get to some node (from the
  starting node). Evaluate it to a printable expression. For now store the text
  representation a solution to the expr. IN-PROGRESS

- Albin: Use capstone-rs to disassemble a slice of bytes. Later on this
  will be integrated within the GUI disassembly view. IN-PROGRESS.

- Linus: **REPORT** Symbolic input & state forking, evaluation section.
  IN-PROGRESS.

- Clara: Represent compressed nodes as an ordered list of original nodes. This
  will help in the implementation of compression algorithm and also be needed
  to reconstruct the disassembly of compressed nodes on the GUI side.
  IN-PROGRESS.

- Loke: **GRAPH RENDERING** LOW PRIO Improve time complexity using
  Barnes-Hut/Quadtree, currently ≈ n^3 in number of nodes.
- Loke: **REPORT** Feature set section, discuss s2e vs symQEMU vs angr vs symCC 
- Loke: include the computed native addr in CFG node as metadata.
  NO-PROGRESS. Implemented in 70 which is blocked by 78.

- Samuel: **REPORT** Describe CFG compression a little bit in the graph, why and
  how. Strongly connected components. IN-PROGRESS.
- Samuel: Rework PR 69 to be more in line with existing setup expecially
  regarding the FFI boundary. Split into 77 (reviewable) and 78
  (IN-PROGRESS).

- UNASSIGNED: create a subcommand that creates a recipe json with no symbolic
  input. NO-PROGRESS. Maybe we just need an example file one can copy and edit.
- UNASSIGNED: create an example recipe file with all features displayed and
  commented (documented). NO-PROGRESS.
- UNASSIGNED: Send over instruction slice for a node when it is new. Send this
  only every so often. NO-PROGRESS.
- UNASSIGNED: Debug libamba code. Libamba (bootleg valgrind
  reimplementation) is currently broken and needs to be debugged to figure out
  what's wrong. NO-PROGRESS.
- UNASSIGNED: Rewrite recipe crate to do more syntax and semantic checks to
  catch more syntactically correct but semantically incorrect recipe files.
  Example: that the binary is statically linked etc.
- UNASSIGNED: CGC binaries. NO PROGRESS
- UNASSIGNED: GUI tool to choose to prioritize a certain "state" in S2E. So
  that one can focus on a branch that is intresting and ignore others manually.
  This will give more control to the user. Scheduele a state to run later?
- UNASSIGNED: Monitor syscalls made by the binary (hook on `onSyscall`? Is
  there such a thing?).
- UNASSIGNED: Figure out how to do useful state-merging using S2E. Maybe we
  could do something like creating breakpoints for start and end addresses the
  user has chosen or do something automatic if possible, can this be done using
  S2E with some hooks?. Investigation task.

## TODOS (also refer to backlog)

# Meeting 2023-04-18

## Iulia
- before \cite put a ~, a non-breakable, uniformly sized, blankspace.
- alot of discussion but to summirize, it doesn't feel motivated as the state
  of the project is now, most important to maybe add something easy that makes
  it motivated. Motivation for the project is the most important part.

## Discussion
- Rapport om visualiserad och interaktiv symbolisk fuzzing. Inkluderar en
  prototyp. Denna kategori av mjukvara är användbar då den kombinerar datorns
  fördelar i beräkningskraft (se vanlig fuzzing) med människans fördelar i
  intuition (se decompiler). Exempel på funktionalitet som kombinerar dessa är
  Guided Searching. Människan väljer vilja states som ska prioriteras. Detta
  hanterar ovanstående program, som inte hanteras bra av S2E CUPASearcher

## DONES

## Backlog
- Enaya: **REPORT** debug data, limitations section. NO-PROGRESS.
- Enaya: Store KLEE expressions as metadata to get to some node (from the
  starting node). Evaluate it to a printable expression. For now store the text
  representation a solution to the expr. IN-PROGRESS

- Albin: Use capstone-rs to disassemble a slice of bytes. Later on this
  will be integrated within the GUI disassembly view. IN-PROGRESS.

- Linus: **REPORT** Symbolic input & state forking, evaluation section.
  IN-PROGRESS.

- Clara: Represent compressed nodes as an ordered list of original nodes. This
  will help in the implementation of compression algorithm and also be needed
  to reconstruct the disassembly of compressed nodes on the GUI side.
  IN-PROGRESS.

- Loke: **GRAPH RENDERING** LOW PRIO Improve time complexity using
  Barnes-Hut/Quadtree, currently ≈ n^3 in number of nodes.
- Loke: **REPORT** Feature set section, discuss s2e vs symQEMU vs angr vs symCC 
- Loke: include the computed native addr in CFG node as metadata.
  NO-PROGRESS. Implemented in 70 which is blocked by 78.

- Samuel: **REPORT** Describe CFG compression a little bit in the graph, why and
  how. Strongly connected components. IN-PROGRESS.
- Samuel: Rework PR 69 to be more in line with existing setup expecially
  regarding the FFI boundary. Split into 77 (reviewable) and 78
  (IN-PROGRESS).

- UNASSIGNED: create a subcommand that creates a recipe json with no symbolic
  input. NO-PROGRESS. Maybe we just need an example file one can copy and edit.
- UNASSIGNED: create an example recipe file with all features displayed and
  commented (documented). NO-PROGRESS.
- UNASSIGNED: Send over instruction slice for a node when it is new. Send this
  only every so often. NO-PROGRESS.
- UNASSIGNED: Debug libamba code. Libamba (bootleg valgrind
  reimplementation) is currently broken and needs to be debugged to figure out
  what's wrong. NO-PROGRESS.
- UNASSIGNED: Rewrite recipe crate to do more syntax and semantic checks to
  catch more syntactically correct but semantically incorrect recipe files.
  Example: that the binary is statically linked etc.
- UNASSIGNED: CGC binaries. NO PROGRESS
- UNASSIGNED: GUI tool to choose to prioritize a certain "state" in S2E. So
  that one can focus on a branch that is intresting and ignore others manually.
  This will give more control to the user. Scheduele a state to run later?
- UNASSIGNED: Monitor syscalls made by the binary (hook on `onSyscall`? Is
  there such a thing?).
- UNASSIGNED: Figure out how to do useful state-merging using S2E. Maybe we
  could do something like creating breakpoints for start and end addresses the
  user has chosen or do something automatic if possible, can this be done using
  S2E with some hooks?. Investigation task.
- UNASSIGNED: Implement a "Searcher" that has ability to be controlled by the
  user.

## TODOS (also refer to backlog)
- Linus: **REPORT** Omformulera syfte att passa det vi vill skriva om.

# Meeting 2023-04-21

## Iulia
- Find a new good title. Something along the lines of "AMBA: a tool for ..."

## Discussion

## DONES
- Samuel: Rework PR 69 to be more in line with existing setup expecially
  regarding the FFI boundary. Split into 77 (reviewable) and 78.
- Samuel: Draw part 3 - Refactor IPC to include metadata about in the message
  to the GUI.

## Backlog
- Enaya: **REPORT** debug data, limitations section. NO-PROGRESS.
- Enaya: Store KLEE expressions as metadata to get to some node (from the
  starting node). Evaluate it to a printable expression. For now store the text
  representation a solution to the expr. IN-PROGRESS

- Albin: Use capstone-rs to disassemble a slice of bytes. Later on this
  will be integrated within the GUI disassembly view. IN-PROGRESS.

- Linus: **REPORT** Symbolic input & state forking, evaluation section.
  IN-PROGRESS.
- Linus: **REPORT** Omformulera syfte att passa det vi vill skriva om.
  IN-PROGRESS.

- Clara: Represent compressed nodes as an ordered list of original nodes. This
  will help in the implementation of compression algorithm and also be needed
  to reconstruct the disassembly of compressed nodes on the GUI side.
  IN-PROGRESS.

- Loke: **GRAPH RENDERING** LOW PRIO Improve time complexity using
  Barnes-Hut/Quadtree, currently ≈ n^3 in number of nodes.
- Loke: **REPORT** Feature set section, discuss s2e vs symQEMU vs angr vs symCC 
- Loke: include the computed native addr in CFG node as metadata.
  NO-PROGRESS. Implemented in 70 which is blocked by 78.

- Samuel: **REPORT** Describe CFG compression a little bit in the graph, why and
  how. Strongly connected components. IN-PROGRESS.
- Samuel: Implement a "Searcher" that has ability to be controlled by the
  user.

- UNASSIGNED: create a subcommand that creates a recipe json with no symbolic
  input. NO-PROGRESS. Maybe we just need an example file one can copy and edit.
- UNASSIGNED: create an example recipe file with all features displayed and
  commented (documented). NO-PROGRESS.
- UNASSIGNED: Send over instruction slice for a node when it is new. Send this
  only every so often. NO-PROGRESS.
- UNASSIGNED: Debug libamba code. Libamba (bootleg valgrind
  reimplementation) is currently broken and needs to be debugged to figure out
  what's wrong. NO-PROGRESS.
- UNASSIGNED: Rewrite recipe crate to do more syntax and semantic checks to
  catch more syntactically correct but semantically incorrect recipe files.
  Example: that the binary is statically linked etc.
- UNASSIGNED: CGC binaries. NO PROGRESS
- UNASSIGNED: GUI tool to choose to prioritize a certain "state" in S2E. So
  that one can focus on a branch that is intresting and ignore others manually.
  This will give more control to the user. Scheduele a state to run later?
- UNASSIGNED: Monitor syscalls made by the binary (hook on `onSyscall`? Is
  there such a thing?).
- UNASSIGNED: Figure out how to do useful state-merging using S2E. Maybe we
  could do something like creating breakpoints for start and end addresses the
  user has chosen or do something automatic if possible, can this be done using
  S2E with some hooks?. Investigation task.

## TODOS (also refer to backlog)
- Enaya: **REPORT** Fix \cite not having ~ before it according to Iulias
  feedback.
- Samuel: Move graph compression to GUI (so multiprocessing is available) side
  and add options to view different graphs (symbolic vs CFG vs compressed
  symbolic vs compressed CFG) in the GUI. (Reviewable).
- Samuel: Send incremental data over IPC to GUI and build the graphs on the GUI
  side. Before we were building the graph in the S2E process and sending a
  whole graph over IPC. Store the "diff" in S2E side and eventually send over
  everything to the GUI side over IPC.
- Loke: Embedding (layout in the 2D plane) subgraph (Non-sibling subset of
  nodes, ie. parents and children, graph of all paths that also include "this
  node"). It is computationally heavy to render the entire graph as it
  will be too huge for larger programs.
- UNASSIGNED: Highlight nodes with self-link with a unicode circular arrow
- UNASSIGNED: Combine dissembly + debugdata and show in the GUI.
- Albin+Clara: Expand and write more in Existerande verktyg section.

## Discussion about Report
Existerande verktyg should maybe mention
- DETAILED: Angr
- DETAILED: Ghidra
- MENTION: AFL++
- MENTION: S2E ootb
- MENTION: Binary ninja
- MENTION: SymQEMU
- ~~SEESAW?~~
- ~~X-force~~
- MENTION: SAGE? Microsoft internal fuzzer. Mentioned here at least:
  https://link.springer.com/referenceworkentry/10.1007/978-981-15-6401-7_40-1
- MENTION: Symbolic Debugging (Java)? https://www.key-project.org/applications/debugging/#Literature


Talk about a subset of these tools in the introduction as well.


# Meeting 2023-04-27

## Report progress
- Linus: symbolic execution engine almost done
- Linus: done with purpose section
- Samuel: Skeleton for implementation section done, spell it out
- Albin+Clara: Existerande verktyg section - Partially done.
- Limitations, Amba left to write.
- More references needed in many places. About symbolic execution, and other
  places in Theory section...


## DONES
- Enaya: **REPORT** Fix \cite not having ~ before it according to Iulias
  feedback.
- Linus: **REPORT** Omformulera syfte att passa det vi vill skriva om.
  IN-PROGRESS.
- Samuel: Move graph compression to GUI (so multiprocessing is available) side
  and add options to view different graphs (symbolic vs CFG vs compressed
  symbolic vs compressed CFG) in the GUI. (Reviewable).
- Samuel: Send incremental data over IPC to GUI and build the graphs on the GUI
  side. Before we were building the graph in the S2E process and sending a
  whole graph over IPC. Store the "diff" in S2E side and eventually send over
  everything to the GUI side over IPC.

## Backlog
- Enaya: **REPORT** debug data, limitations section. NO-PROGRESS.
- Enaya: Store KLEE expressions as metadata to get to some node (from the
  starting node). Evaluate it to a printable expression. For now store the text
  representation a solution to the expr. IN-PROGRESS

- Albin: Use capstone-rs to disassemble a slice of bytes. Later on this
  will be integrated within the GUI disassembly view. IN-PROGRESS.
- Albin+Clara: Expand and write more in Existerande verktyg section. IN-PROGRESS.

- Linus: **REPORT** Symbolic input & state forking, evaluation section.
  IN-PROGRESS.

- Clara: Represent compressed nodes as an ordered list of original nodes. This
  will help in the implementation of compression algorithm and also be needed
  to reconstruct the disassembly of compressed nodes on the GUI side.
  IN-PROGRESS.

- Loke: **GRAPH RENDERING** LOW PRIO Improve time complexity using
  Barnes-Hut/Quadtree, currently ≈ n^3 in number of nodes.
- Loke: **REPORT** Feature set section, discuss s2e vs symQEMU vs angr vs symCC 
- Loke: include the computed native addr in CFG node as metadata.
  NO-PROGRESS. Implemented in 70 which is blocked by 78.
- Loke: Embedding (layout in the 2D plane) subgraph (Non-sibling subset of
  nodes, ie. parents and children, graph of all paths that also include "this
  node"). It is computationally heavy to render the entire graph as it
  will be too huge for larger programs.

- Samuel: **REPORT** Describe CFG compression a little bit in the graph, why and
  how. Strongly connected components. IN-PROGRESS.
- Samuel: Implement a "Searcher" that has ability to be controlled by the
  user. IN-PROGRESS.

- UNASSIGNED: create a subcommand that creates a recipe json with no symbolic
  input. NO-PROGRESS. Maybe we just need an example file one can copy and edit.
- UNASSIGNED: create an example recipe file with all features displayed and
  commented (documented). NO-PROGRESS.
- UNASSIGNED: Send over instruction slice for a node when it is new. Send this
  only every so often. NO-PROGRESS.
- UNASSIGNED: Debug libamba code. Libamba (bootleg valgrind
  reimplementation) is currently broken and needs to be debugged to figure out
  what's wrong. NO-PROGRESS.
- UNASSIGNED: Rewrite recipe crate to do more syntax and semantic checks to
  catch more syntactically correct but semantically incorrect recipe files.
  Example: that the binary is statically linked etc.
- UNASSIGNED: CGC binaries. NO PROGRESS
- UNASSIGNED: GUI tool to choose to prioritize a certain "state" in S2E. So
  that one can focus on a branch that is intresting and ignore others manually.
  This will give more control to the user. Scheduele a state to run later?
- UNASSIGNED: Monitor syscalls made by the binary (hook on `onSyscall`? Is
  there such a thing?).
- UNASSIGNED: Figure out how to do useful state-merging using S2E. Maybe we
  could do something like creating breakpoints for start and end addresses the
  user has chosen or do something automatic if possible, can this be done using
  S2E with some hooks?. Investigation task.
- UNASSIGNED: Highlight nodes with self-link with a unicode circular arrow
- UNASSIGNED: Combine dissembly + debugdata and show in the GUI.

## TODOS (also refer to backlog)

## Discussion about Report
Existerande verktyg should maybe mention
- DETAILED: Angr
- DETAILED: Ghidra
- MENTION: AFL++
- MENTION: S2E ootb
- MENTION: Binary ninja
- MENTION: SymQEMU
- ~~SEESAW?~~
- ~~X-force~~
- MENTION: SAGE? Microsoft internal fuzzer. Mentioned here at least:
  https://link.springer.com/referenceworkentry/10.1007/978-981-15-6401-7_40-1
- MENTION: Symbolic Debugging (Java)? https://www.key-project.org/applications/debugging/#Literature


Talk about a subset of these tools in the introduction as well.



# Meeting 2023-05-02

## Discussion
- We should focus on report mainly, project and report except presentation
  should be done in 2 weeks.
- More references needed in many places. About symbolic execution, and other
  places in Theory section...
- We should clean up parts of report that isn't needed or relevant anymore,
  e.g. Metod section under Evaluation section? Maybe can keep parts of it under
  some other section title.
- PRIO är Evaluation section, Inledning sammanhållning, få ner mer text i
  rapporten för att senare arbeta och göra bättre.

## DONES
- Albin+Clara: Expand and write more in Existerande verktyg section. IN-PROGRESS.
- Loke: **GRAPH RENDERING** LOW PRIO Improve time complexity using
  Barnes-Hut/Quadtree, currently ≈ n^3 in number of nodes.
- Loke: Embedding (layout in the 2D plane) subgraph (Non-sibling subset of
  nodes, ie. parents and children, graph of all paths that also include "this
  node"). It is computationally heavy to render the entire graph as it
  will be too huge for larger programs.

## Backlog
- Enaya: **REPORT** debug data. NO-PROGRESS.
- Enaya: Store KLEE expressions as metadata to get to some node (from the
  starting node). Evaluate it to a printable expression. For now store the text
  representation a solution to the expr. IN-PROGRESS

- Albin: Use capstone-rs to disassemble a slice of bytes. Later on this
  will be integrated within the GUI disassembly view. Cleanup a little, return
  result. IN-PROGRESS.

- Linus: **REPORT** Symbolic input & state forking (under section 5).
- Linus: **REPORT** Evaluation section. (Jämföra med andra verktyg, exempelvis
  CFG:en med Ghidras CFG av samma program. Kanske även analysera någon
  exempelprogram med AMBA.)

- Clara: Represent compressed nodes as an ordered list of original nodes. This
  will help in the implementation of compression algorithm and also be needed
  to reconstruct the disassembly of compressed nodes on the GUI side.
  IN-PROGRESS.

- Loke: **REPORT** Feature set section, discuss s2e vs symQEMU vs angr vs symCC
  i AMBA section instead of Inledning.
- Loke: include the computed native addr in CFG node as metadata.
  NO-PROGRESS.
- Loke: Send over instruction slice for a node when it is new. Send this
  only every so often. NO-PROGRESS.
- Loke: Combine dissembly + debugdata and show in the GUI.
- Loke: Highlight nodes with self-link with a unicode circular arrow.

- Samuel: **REPORT** Describe CFG compression a little bit in the graph, why and
  how. Strongly connected components. IN-PROGRESS.
- Samuel: Implement a "Searcher" that has ability to be controlled by the
  user. IN-PROGRESS.

- UNASSIGNED: create a subcommand that creates a recipe json with no symbolic
  input. NO-PROGRESS. Maybe we just need an example file one can copy and edit.
- UNASSIGNED: create an example recipe file with all features displayed and
  commented (documented). NO-PROGRESS.
- UNASSIGNED: Rewrite recipe crate to do more syntax and semantic checks to
  catch more syntactically correct but semantically incorrect recipe files.
  Example: that the binary is statically linked etc.
- UNASSIGNED: Monitor syscalls made by the binary (hook on `onSyscall`? Is
  there such a thing?).
- UNASSIGNED: Figure out how to do useful state-merging using S2E. Maybe we
  could do something like creating breakpoints for start and end addresses the
  user has chosen or do something automatic if possible, can this be done using
  S2E with some hooks?. Investigation task.

## TODOS (also refer to backlog)
- Enaya: **REPORT** Gör inledningen mer mer sammanhållet och gör läsaren redo
  för att förstå syftet. Ge en röd tråd genom hela section.
- Enaya: **REPORT** Skriv om "tidigare arbeten" section under inledning till att
  handla om existerande verktyg och inte seesaw och xforece. Inte detaljerat,
  referera mest till senare section om existerande verktyg.
- Clara: **REPORT** Existerande verktyg section: Talk about automatic
  fuzzers like AFL++ under another subsection that is not "dynamiska
  binäranalysramverk".
- Clara: **REPORT** Add more references About symbolic execution.
- Samuel: Bloody Colour nodes after strongly connected components and
  functions, mate. Howdy, yeehaw.

- UNASSIGNED: **REPORT** Slutsats section: Discuss a little about s2e usage and
  what was hard and about the nix environment.
- UNASSIGNED: More references needed in many places. places in Theory
  section... Kan göras lite senare.

# Meeting 2023-05-05

## Discussion
- Linus not available 18-23 maj.
- Video presentation needs to be done by 17th maj.

- We should focus on report mainly, project and report except presentation
  should be done in 2 weeks.
- We should clean up parts of report that isn't needed or relevant anymore,
  e.g. Metod section under Evaluation section? Maybe can keep parts of it under
  some other section title.
- PRIO är Evaluation section, Inledning sammanhållning, få ner mer text i
  rapporten för att senare arbeta och göra bättre.

## DONES
- Albin: Use capstone-rs to disassemble a slice of bytes. Later on this
  will be integrated within the GUI disassembly view. Cleanup a little, return
  result.
- Loke: Mark nodes with self-link with a unicode circular arrow.
- Samuel: **REPORT** Describe CFG compression a little bit in the graph, why
  and how. Strongly connected components.

## Backlog
- Enaya: **REPORT** debug data. NO-PROGRESS.
- Enaya: Store KLEE expressions as metadata to get to some node (from the
  starting node). Evaluate it to a printable expression. For now store the text
  representation a solution to the expr. IN-PROGRESS
- Enaya: **REPORT** Gör inledningen mer mer sammanhållet och gör läsaren redo
  för att förstå syftet. Ge en röd tråd genom hela section. IN-PROGRESS.
- Enaya: **REPORT** Skriv om "tidigare arbeten" section under inledning till
  att handla om existerande verktyg och inte seesaw och xforece. Inte
  detaljerat, referera mest till senare section om existerande verktyg.
  NO-PROGRESS.

- Linus: **REPORT** Symbolic input & state forking (under section 5).
  NO-PROGRESS.
- Linus: **REPORT** Evaluation section. (Jämföra med andra verktyg, exempelvis
  CFG:en med Ghidras CFG av samma program. Kanske även analysera någon
  exempelprogram med AMBA.) NO-PROGRESS.

- Clara: Represent compressed nodes as an ordered list of original nodes. This
  will help in the implementation of compression algorithm and also be needed
  to reconstruct the disassembly of compressed nodes on the GUI side.
  IN-PROGRESS.
- Clara: **REPORT** Existerande verktyg section: Talk about automatic
  fuzzers like AFL++ under another subsection that is not "dynamiska
  binäranalysramverk". IN-PROGRESS.
- Clara: **REPORT** Add more references About symbolic execution. IN-PROGRESS.

- Loke: **REPORT** Feature set section, discuss s2e vs symQEMU vs angr vs symCC
  i AMBA section instead of Inledning. NO-PROGRESS.
- Loke: include the computed native addr in CFG node as metadata.
  IN-PROGRESS.
- Loke: Send over instruction slice for a node when it is new. Send this only
  every so often. IN-PROGRESS.
- Loke: Combine dissembly + debugdata and show in the GUI. NO-PROGRESS.

- Samuel: Implement a "Searcher" that has ability to be controlled by the
  user. IN-PROGRESS.
- Samuel: Bloody Colour nodes after strongly connected components and
  functions, mate. Howdy, yeehaw. NO-PROGRESS.

- UNASSIGNED: create a subcommand that creates a recipe json with no symbolic
  input. NO-PROGRESS. Maybe we just need an example file one can copy and edit.
- UNASSIGNED: create an example recipe file with all features displayed and
  commented (documented). NO-PROGRESS.
- UNASSIGNED: Rewrite recipe crate to do more syntax and semantic checks to
  catch more syntactically correct but semantically incorrect recipe files.
  Example: that the binary is statically linked etc.
- UNASSIGNED: Monitor syscalls made by the binary (hook on `onSyscall`? Is
  there such a thing?).
- UNASSIGNED: Figure out how to do useful state-merging using S2E. Maybe we
  could do something like creating breakpoints for start and end addresses the
  user has chosen or do something automatic if possible, can this be done using
  S2E with some hooks?. Investigation task.
- UNASSIGNED: **REPORT** Slutsats section: Discuss a little about s2e usage and
  what was hard and about the nix environment.
- UNASSIGNED: More references needed in many places. places in Theory
  section... Kan göras lite senare.

## TODOS (also refer to backlog)
- Linus: **REPORT** Write about symbolic fuzzing in theory section.
- Albin: **REPORT** Fix begreppslista according to Supervisor feedback.
- Enaya: **REPORT** Fix title+cover+copyright pages according to Supervisor
  feedback and turn them swedish.
- Clara: **REPORT** Write Abstract + Sammanfattning.
- UNASSIGNED: **LOW_PRIO** Document Recipe file/json format.
- UNASSIGNED: Only process (run layout algorithm) currently viewed graph in GUI.

# Meeting 2023-05-09

## Discussion
- Video presentation needs to be done by 17th may.
- Fackspråkshandledning next week Tuesday 16th may, todo check when report has
  to be submitted.
- Report submission next week Monday 15th may.

- We should focus on report mainly, project and report except presentation
  should be done in 2 weeks.
- We should clean up parts of report that isn't needed or relevant anymore,
  e.g. Metod section under Evaluation section? Maybe can keep parts of it under
  some other section title.
- PRIO är Evaluation section, Inledning sammanhållning, få ner mer text i
  rapporten för att senare arbeta och göra bättre.
- Handle feedback from Iulia on chapter 2

## DONES
- Enaya: **REPORT** Fix title+cover+copyright pages according to Supervisor
  feedback and turn them swedish.
- Clara: **REPORT** Existerande verktyg section: Talk about automatic
  fuzzers like AFL++ under another subsection that is not "dynamiska
  binäranalysramverk". IN-PROGRESS.
- Clara: **REPORT** Add more references About symbolic execution. IN-PROGRESS.
- Loke: include the computed native addr in CFG node as metadata.
  IN-PROGRESS.
- Loke: Send over instruction slice for a node when it is new. Send this only
  every so often. IN-PROGRESS.

## Backlog
- Enaya: **REPORT** debug data. NO-PROGRESS.
- Enaya: Store KLEE expressions as metadata to get to some node (from the
  starting node). Evaluate it to a printable expression. For now store the text
  representation a solution to the expr. IN-PROGRESS.
- Enaya: **REPORT** Gör inledningen mer mer sammanhållet och gör läsaren redo
  för att förstå syftet. Ge en röd tråd genom hela section. IN-PROGRESS.
- Enaya: **REPORT** Skriv om "tidigare arbeten" section under inledning till
  att handla om existerande verktyg och inte seesaw och xforece. Inte
  detaljerat, referera mest till senare section om existerande verktyg.
  NO-PROGRESS.

- ~~Linus~~UNASSIGNED: **REPORT** Symbolic input & state forking (under section 5).
  NO-PROGRESS.
- Linus: **REPORT** Evaluation section. (Jämföra med andra verktyg, exempelvis
  CFG:en med Ghidras CFG av samma program. Kanske även analysera någon
  exempelprogram med AMBA.) IN-PROGRESS.
- Linus: **REPORT** Write about symbolic fuzzing in theory section. IN-PROGRESS.

- Clara: Represent compressed nodes as an ordered list of original nodes. This
  will help in the implementation of compression algorithm and also be needed
  to reconstruct the disassembly of compressed nodes on the GUI side.
  IN-PROGRESS.
- Clara: **REPORT** Write Abstract + Sammanfattning.

- Albin: **REPORT** Fix begreppslista according to Supervisor feedback.
  IN-PROGRESS.

- Loke: **REPORT** Feature set section, discuss s2e vs symQEMU vs angr vs symCC
  i AMBA section instead of Inledning. NO-PROGRESS.
- Loke: Combine dissembly + debugdata and show in the GUI, split into two PRs.
  IN-PROGRESS.

- Samuel: Implement a "Searcher" that has ability to be controlled by the
  user, reviewable, potential theoretical race condition in the searcher code.
  IN-PROGRESS.
- Samuel: Colour nodes after strongly connected components and functions.
  NO-PROGRESS.

- UNASSIGNED: **REPORT** Slutsats section: Discuss a little about s2e usage and
  what was hard and about the nix environment.
- UNASSIGNED: More references needed in many places. places in Theory
  section... Kan göras lite senare.
- UNASSIGNED: Only process (run layout algorithm) currently viewed graph in GUI.

### LOW_PRIO:
- UNASSIGNED: create a subcommand that creates a recipe json with no symbolic
  input. NO-PROGRESS. Maybe we just need an example file one can copy and edit.
- UNASSIGNED: create an example recipe file with all features displayed and
  commented (documented). NO-PROGRESS.
- UNASSIGNED: Rewrite recipe crate to do more syntax and semantic checks to
  catch more syntactically correct but semantically incorrect recipe files.
  Example: that the binary is statically linked etc.
- UNASSIGNED: Monitor syscalls made by the binary (hook on `onSyscall`? Is
  there such a thing?).
- UNASSIGNED: Figure out how to do useful state-merging using S2E. Maybe we
  could do something like creating breakpoints for start and end addresses the
  user has chosen or do something automatic if possible, can this be done using
  S2E with some hooks?. Investigation task.
- UNASSIGNED: **LOW_PRIO** Document Recipe file/json format.

## TODOS (also refer to backlog)
- Loke?: Summerize this as previous work, what differentiates us from them? Are
  there simmiliar work to this one?:
  https://ieeexplore.ieee.org/abstract/document/9161524
- Albin: Make S2E architecture graph as a real figure.
- Samuel: Nodidentifiering som glömmer symbolic state, behåller basic block.
  Visar möjliga paths lite för generellt men i mindre graf. A new graph that
  compresses the basic block graph.
- Loke: **BUG_FIX** Fix repulsion-approximation slider blinking active/non-active.


# Meeting 2023-05-12

## Discussion
- Mail to Francisco regarding workshop tomorrow.


## DONES
- Linus: **REPORT** Write about symbolic fuzzing in theory section. IN-PROGRESS.

## Backlog
- Enaya: **REPORT** debug data. NO-PROGRESS.
- Enaya: Store KLEE expressions as metadata to get to some node (from the
  starting node). Evaluate it to a printable expression. For now store the text
  representation a solution to the expr. IN-PROGRESS.
- Enaya: **REPORT** Gör inledningen mer mer sammanhållet och gör läsaren redo
  för att förstå syftet. Ge en röd tråd genom hela section. IN-PROGRESS.
- Enaya: **REPORT** Skriv om "tidigare arbeten" section under inledning till
  att handla om existerande verktyg och inte seesaw och xforece. Inte
  detaljerat, referera mest till senare section om existerande verktyg.
  NO-PROGRESS.

- ~~Linus~~UNASSIGNED: **REPORT** Symbolic input & state forking (under section
  5). NO-PROGRESS.
- Linus: **REPORT** Evaluation section. (Jämföra med andra verktyg, exempelvis
  CFG:en med Ghidras CFG av samma program. Kanske även analysera någon
  exempelprogram med AMBA.) IN-PROGRESS.

- Clara: Represent compressed nodes as an ordered list of original nodes. This
  will help in the implementation of compression algorithm and also be needed
  to reconstruct the disassembly of compressed nodes on the GUI side.
  IN-PROGRESS.
- Clara: **REPORT** Write Abstract + Sammanfattning. IN-PROGRESS.

- Albin: **REPORT** Fix begreppslista according to Supervisor feedback.
  IN-PROGRESS.
- Albin: **REPORT** Make S2E architecture graph as a real figure.

- Loke: **REPORT** Feature set section, discuss s2e vs symQEMU vs angr vs symCC
  i AMBA section instead of Inledning. NO-PROGRESS.
- Loke: Combine dissembly + debugdata and show in the GUI, split into two PRs.
  IN-PROGRESS.
- Loke?: **REPORT** Summerize this as previous work, what differentiates us
  from them? Are there simmiliar work to this one?:
  https://ieeexplore.ieee.org/abstract/document/9161524
- Loke: **BUG_FIX** Fix repulsion-approximation slider blinking
  active/non-active.

- Samuel: Implement a "Searcher" that has ability to be controlled by the
  user, reviewable, potential theoretical race condition in the searcher code.
  IN-PROGRESS.
- Samuel: Colour nodes after strongly connected components and functions.
  NO-PROGRESS.
- Samuel: Nodidentifiering som glömmer symbolic state, behåller basic block.
  Visar möjliga paths lite för generellt men i mindre graf. A new graph that
  compresses the basic block graph.

- UNASSIGNED: **REPORT** Slutsats section: Discuss a little about s2e usage and
  what was hard and about the nix environment.
- UNASSIGNED: More references needed in many places. places in Theory
  section... Kan göras lite senare.
- UNASSIGNED: Only process (run layout algorithm) currently viewed graph in
  GUI.
- Existerande verktyg.

### LOW_PRIO:
- UNASSIGNED: create a subcommand that creates a recipe json with no symbolic
  input. NO-PROGRESS. Maybe we just need an example file one can copy and edit.
- UNASSIGNED: create an example recipe file with all features displayed and
  commented (documented). NO-PROGRESS.
- UNASSIGNED: Rewrite recipe crate to do more syntax and semantic checks to
  catch more syntactically correct but semantically incorrect recipe files.
  Example: that the binary is statically linked etc.
- UNASSIGNED: Monitor syscalls made by the binary (hook on `onSyscall`? Is
  there such a thing?).
- UNASSIGNED: Figure out how to do useful state-merging using S2E. Maybe we
  could do something like creating breakpoints for start and end addresses the
  user has chosen or do something automatic if possible, can this be done using
  S2E with some hooks?. Investigation task.
- UNASSIGNED: **LOW_PRIO** Document Recipe file/json format.

## TODOS (also refer to backlog)

# Meeting 2023-05-16

# Presentation discussion
- Result oriented
- Introduction: motivation, why symbolic execution and fuzzing and what they
  are.
- Demo
- Prerecorded demo in case live demo goes wrong.

## Extra
- Samuel do demo stuff
- Albin + Linus writing on report
- Enaya, Loke, Clara: presentation writing/preparing

- Abstract is lying? We have nothing to do with memory vulnerbility, it is just
  mentioned because symbolic execution methods could be used and have been used
  in some cases for detection of them. We don't detect memory vulnerbility we
  just allow symbolic execution and visualize it and let the user do state
  prioritization. And show the progress etc.
- If not then don't mention it - Iulia?


## DONES
- Linus: **REPORT** Write about symbolic fuzzing in theory section. IN-PROGRESS.
- Enaya: **REPORT** Gör inledningen mer mer sammanhållet och gör läsaren redo
  för att förstå syftet. Ge en röd tråd genom hela section.
- Enaya: **REPORT** Skriv om "tidigare arbeten" section under inledning till
  att handla om existerande verktyg och inte seesaw och xforece. Inte
  detaljerat, referera mest till senare section om existerande verktyg.
- Enaya: **REPORT** Skrivit om delar i teori och lagt till delar.
- ~~Linus~~Loke?: **REPORT** Symbolic input & state forking (under section
  5). 
- Clara: **REPORT** Write Abstract + Sammanfattning.
- Linus: **REPORT** Evaluation section. (Jämföra med andra verktyg, exempelvis
  CFG:en med Ghidras CFG av samma program. Kanske även analysera någon
  exempelprogram med AMBA.)
- Albin: **REPORT** Fix begreppslista according to Supervisor feedback.
- Loke: **REPORT** Feature set section, discuss s2e vs symQEMU vs angr vs symCC
  i AMBA section instead of Inledning.
- Loke: Combine dissembly + debugdata and show in the GUI, split into two PRs.
- Loke?: **REPORT** Summerize this as previous work, what differentiates us
  from them? Are there simmiliar work to this one?:
  https://ieeexplore.ieee.org/abstract/document/9161524
- Samuel: Implement a "Searcher" that has ability to be controlled by the
  user, reviewable, potential theoretical race condition in the searcher code.
- Albin: **REPORT** Slutsats section: Discuss a little about s2e usage and
  what was hard and about the nix environment.
- everyone: More references needed in many places. places in Theory
  section... Kan göras lite senare.

- Enaya: **REPORT** debug data. NO-PROGRESS. IGNORED.
- Enaya: Store KLEE expressions as metadata to get to some node (from the
  starting node). Evaluate it to a printable expression. For now store the text
  representation a solution to the expr. IN-PROGRESS. IGNORING TASK.

## Backlog

- Clara: Represent compressed nodes as an ordered list of original nodes. This
  will help in the implementation of compression algorithm and also be needed
  to reconstruct the disassembly of compressed nodes on the GUI side.
  IN-PROGRESS.

- Albin: **REPORT** Make S2E architecture graph as a real figure.

- Loke: **BUG_FIX** Fix repulsion-approximation slider blinking
  active/non-active.

- Samuel: Colour nodes after strongly connected components and functions.
  NO-PROGRESS.
- Samuel: Nodidentifiering som glömmer symbolic state, behåller basic block.
  Visar möjliga paths lite för generellt men i mindre graf. A new graph that
  compresses the basic block graph.

- UNASSIGNED: Only process (run layout algorithm) currently viewed graph in
  GUI.

### LOW_PRIO:
- UNASSIGNED: create a subcommand that creates a recipe json with no symbolic
  input. NO-PROGRESS. Maybe we just need an example file one can copy and edit.
- UNASSIGNED: create an example recipe file with all features displayed and
  commented (documented). NO-PROGRESS.
- UNASSIGNED: Rewrite recipe crate to do more syntax and semantic checks to
  catch more syntactically correct but semantically incorrect recipe files.
  Example: that the binary is statically linked etc.
- UNASSIGNED: Monitor syscalls made by the binary (hook on `onSyscall`? Is
  there such a thing?).
- UNASSIGNED: Figure out how to do useful state-merging using S2E. Maybe we
  could do something like creating breakpoints for start and end addresses the
  user has chosen or do something automatic if possible, can this be done using
  S2E with some hooks?. Investigation task.
- UNASSIGNED: **LOW_PRIO** Document Recipe file/json format.

## TODOS (also refer to backlog)
- Everone needs to write a page for Medverkansrapport
- We need to create a video presentaion



# Meeting 2023-05-19

## Möten nästa vecka:

| Day     | Description + Dat                    | Time         |
|---------|--------------------------------------|--------------|
| Måndag  | Möte 22/5                            | 13:15        |
| Torsdag | Möte 25/5                            | 10:00        |
| Fredag  | Presentation+muntlig opponering 26/5 | 9:30 - 12:00 |


## DONES
- Enaya: Send over symoblic solutions for all states to GUI.
- All: Everone needs to write a page for Medverkansrapport.
- Samuel: Create a video presentation.
- Samuel: Colour nodes after strongly connected components, states, and
  ~~functions~~.
- Loke: **BUG_FIX** Fix repulsion-approximation slider blinking
  active/non-active.
- Loke: Display symbolic solutions in GUI.
- Loke: Make graph converge faster?
- Loke: Only process (run layout algorithm) currently viewed graph in GUI.

## Backlog

- Clara: Represent compressed nodes as an ordered list of original nodes. This
  will help in the implementation of compression algorithm and also be needed
  to reconstruct the disassembly of compressed nodes on the GUI side.
  IN-PROGRESS.

- Albin: **REPORT** Make S2E architecture graph as a real figure. IN-PROGRESS.

- Samuel: Nodidentifiering som glömmer symbolic state, behåller basic block.
  Visar möjliga paths lite för generellt men i mindre graf. A new graph that
  compresses the basic block graph.

### LOW_PRIO:
- UNASSIGNED: create a subcommand that creates a recipe json with no symbolic
  input. NO-PROGRESS. Maybe we just need an example file one can copy and edit.
- UNASSIGNED: create an example recipe file with all features displayed and
  commented (documented). NO-PROGRESS.
- UNASSIGNED: Rewrite recipe crate to do more syntax and semantic checks to
  catch more syntactically correct but semantically incorrect recipe files.
  Example: that the binary is statically linked etc.
- UNASSIGNED: Monitor syscalls made by the binary (hook on `onSyscall`? Is
  there such a thing?).
- UNASSIGNED: Figure out how to do useful state-merging using S2E. Maybe we
  could do something like creating breakpoints for start and end addresses the
  user has chosen or do something automatic if possible, can this be done using
  S2E with some hooks?. Investigation task.
- UNASSIGNED: **LOW_PRIO** Document Recipe file/json format.

## TODOS (also refer to backlog)
- Loke: **BUG_FIX** Freeze bug sometimes (very often on Enayas computer)
  the whole GUI freezes when a node is selected.
- Samuel: Color nodes by functions.

# Meeting 2023-05-22

## Möten denna vecka

| Day     | Description + Dat                    | Time         |
|---------|--------------------------------------|--------------|
| Torsdag | Möte 25/5                            | 10:00        |
| Fredag  | Presentation+muntlig opponering 26/5 | 9:30 - 12:00 |

## DONES
- Loke+Enaya+Clara+Samuel: Prepare slides and script for presentation.
- Loke+Samuel: Send separate oppositions to the corresponding group.

## Backlog

- Clara: Represent compressed nodes as an ordered list of original nodes. This
  will help in the implementation of compression algorithm and also be needed
  to reconstruct the disassembly of compressed nodes on the GUI side.
  IN-PROGRESS.

- Albin: **REPORT** Make S2E architecture graph as a real figure. IN-PROGRESS.

- Samuel: Nodidentifiering som glömmer symbolic state, behåller basic block.
  Visar möjliga paths lite för generellt men i mindre graf. A new graph that
  compresses the basic block graph.
- Samuel: Color nodes by functions.

- Loke: **BUG_FIX** Freeze bug sometimes (very often on Enayas computer)
  the whole GUI freezes when a node is selected.

### LOW_PRIO:
- UNASSIGNED: create a subcommand that creates a recipe json with no symbolic
  input. NO-PROGRESS. Maybe we just need an example file one can copy and edit.
- UNASSIGNED: create an example recipe file with all features displayed and
  commented (documented). NO-PROGRESS.
- UNASSIGNED: Rewrite recipe crate to do more syntax and semantic checks to
  catch more syntactically correct but semantically incorrect recipe files.
  Example: that the binary is statically linked etc.
- UNASSIGNED: Monitor syscalls made by the binary (hook on `onSyscall`? Is
  there such a thing?).
- UNASSIGNED: Figure out how to do useful state-merging using S2E. Maybe we
  could do something like creating breakpoints for start and end addresses the
  user has chosen or do something automatic if possible, can this be done using
  S2E with some hooks?. Investigation task.
- UNASSIGNED: **LOW_PRIO** Document Recipe file/json format.

## TODOS (also refer to backlog)

# Meeting 2023-05-30

## DONES (updates, DONE quite a while now)
- Loke: **BUG_FIX** Freeze bug sometimes (very often on Enayas computer)
  the whole GUI freezes when a node is selected.
- Albin: **REPORT** Make S2E architecture graph as a real figure.
- Samuel: Nodidentifiering som glömmer symbolic state, behåller basic block.
  Visar möjliga paths lite för generellt men i mindre graf. A new graph that
  compresses the basic block graph.

## Backlog
- Clara: Represent compressed nodes as an ordered list of original nodes. This
  will help in the implementation of compression algorithm and also be needed
  to reconstruct the disassembly of compressed nodes on the GUI side.
  IN-PROGRESS.
- Samuel: Color nodes by functions.


### LOW_PRIO:
- UNASSIGNED: create a subcommand that creates a recipe json with no symbolic
  input. NO-PROGRESS. Maybe we just need an example file one can copy and edit.
- UNASSIGNED: create an example recipe file with all features displayed and
  commented (documented). NO-PROGRESS.
- UNASSIGNED: Rewrite recipe crate to do more syntax and semantic checks to
  catch more syntactically correct but semantically incorrect recipe files.
  Example: that the binary is statically linked etc.
- UNASSIGNED: Monitor syscalls made by the binary (hook on `onSyscall`? Is
  there such a thing?).
- UNASSIGNED: Figure out how to do useful state-merging using S2E. Maybe we
  could do something like creating breakpoints for start and end addresses the
  user has chosen or do something automatic if possible, can this be done using
  S2E with some hooks?. Investigation task.
- UNASSIGNED: **LOW_PRIO** Document Recipe file/json format.

## TODOS (also refer to backlog)
- ALL: Prepare Report for final submission, task details on Discord.
