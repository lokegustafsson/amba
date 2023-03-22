CC ?= gcc
CFLAGS ?= \
	-Wall \
	-Wextra \
	-Wno-unused-parameter \
	-Wconversion \
	-O0 \
	-g \
	-fno-stack-protector \
	-U _FORTIFY_SOURCE \
	$$(APPEND_CFLAGS)

overrun: overrun.c
run_overrun: overrun
	echo  1 | ./overrun
	echo 12 | ./overrun
overrun.S: overrun
	objdump -S overrun > overrun.S

hello: hello.c
hello.S: hello
	objdump -S hello > hello.S

control-flow: control-flow.c
run_control-flow: control-flow
	printf '\x00' | ./control-flow
	printf '\x01' | ./control-flow
	printf '\x02' | ./control-flow
	printf '\x03' | ./control-flow
	printf '\x04' | ./control-flow
	printf '\x05' | ./control-flow
	printf '\x06' | ./control-flow
	printf '\x07' | ./control-flow
	printf '\x08' | ./control-flow
	printf '\x09' | ./control-flow
	printf '\x0a' | ./control-flow
	printf '\x0b' | ./control-flow
	printf '\x0c' | ./control-flow
	printf '\x0d' | ./control-flow
	printf '\x0e' | ./control-flow
	printf '\x0f' | ./control-flow
control-flow.S: control-flow
	objdump -S control-flow > control-flow.S

clean:
	-rm overrun
	-rm overrun.S
	-rm hello
	-rm hello.S
	-rm control-flow
	-rm control-flow.S

.PHONY: clean run_overrun run_control-flow
