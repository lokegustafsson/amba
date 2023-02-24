## Implementation
`autocxx` is a pain to deal with. 
	Subclassing on the rust side is super unreasonable despite it being a supported usecase.
		Reasonable approach is to have the rust code export a C api and call that from C++
	Magic defines were missing (probably originally defined in the build system)
	S2E classes are considered opaque.
		Upstream is working on it, but the open PR is not enough.
		This means that field accessors aren't generated.
	API is very unergonomic: `.to_rust()`, `.pin()` and `.as_ref/mut()` everywhere.
	Fundamentally unsound if qemu or s2e uses any kind of threading. (to create any kind of a rust reference)

## GUI
GTK works, the `relm4` rust wrapper seems reasonable. Also willing to
run in a non-main thread, though it can't move between threads once
initialised. Does not build with current stdenv, though it does on
latest stable.

QT is not ready for rust use yet. Is willing to run in a non-main
thread. Maybe call a python wrapper? Seems a bit overcomplicated.

Web. It's web. It works, there's a tonne of off-the-shelf stuff, but it's still web.
