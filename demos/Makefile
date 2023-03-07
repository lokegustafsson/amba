CC ?= gcc
CFLAGS ?= \
	-O0 \
	-g \
	-fno-stack-protector \
	-U _FORTIFY_SOURCE

overrun: overrun.c
run_overrun: overrun
	echo  1 | ./overrun
	echo 12 | ./overrun
overrun.S: overrun
	objdump -S overrun > overrun.S

hello: hello.c
	musl-gcc -static $(CFLAGS) $^ -o $@
hello.S: hello
	objdump -S hello > hello.S

control-flow: control-flow.c
run_control-flow: control-flow
	./control-flow
	./control-flow a
	./control-flow a a
	./control-flow a a a
	./control-flow a a a a
	./control-flow a a a a a
	./control-flow a a a a a a
	./control-flow a a a a a a a
	./control-flow a a a a a a a a
	./control-flow a a a a a a a a a
	./control-flow a a a a a a a a a a
	./control-flow a a a a a a a a a a a
	./control-flow a a a a a a a a a a a a
control-flow.S: control-flow
	objdump -S control-flow > control-flow.S

clean:
	-rm overrun
	-rm overrun.S
	-rm control-flow
	-rm control-flow.S

.PHONY: clean run_overrun
