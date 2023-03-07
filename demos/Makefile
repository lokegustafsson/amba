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

clean:
	-rm overrun
	-rm overrun.S

.PHONY: clean run_overrun
