CC ?= gcc
CFLAGS ?= \
	-O0 \
	-fno-stack-protector \
	-U _FORTIFY_SOURCE

overrun: overrun.c
run_overrun: overrun
	echo  1 | ./overrun
	echo 12 | ./overrun

clean:
	-rm overrun

.PHONY: clean run_overrun
