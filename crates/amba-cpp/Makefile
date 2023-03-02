CXX := clang++
S2E_DIRS = \
	-I $(S2E_PATH)/klee/include \
	-I $(S2E_PATH)/libcoroutine/include \
	-I $(S2E_PATH)/libcpu/include \
	-I $(S2E_PATH)/libfsigc++/include \
	-I $(S2E_PATH)/libq/include \
	-I $(S2E_PATH)/libs2e/include \
	-I $(S2E_PATH)/libs2ecore/include \
	-I $(S2E_PATH)/libs2eplugins/include \
	-I $(S2E_PATH)/libtcg/include \
	-I $(S2E_PATH)/libvmi/include \
	-I $(S2E_PATH)/tools/lib/X8664BitcodeLibrary
LIBS := \
	-I $(BOOST_PATH) \
	-I $(GLIBC_PATH) \
	-I $(LLVM_PATH) \
	-I $(ZYDIS_PATH)/include \
	-I $(ZYCORE_PATH)/include
ARCHIVES := \
	-L $(ZYDIS_PATH)/lib \
	-l Zydis \
	-L $(ZYCORE_PATH)/lib \
	-l Zycore
CXX-FLAGS := \
	-I inc \
	-std=c++20 \
	$(S2E_DIRS) \
	$(LIBS) \
	$(ARCHIVES) \
	-DBOOST_BIND_GLOBAL_PLACEHOLDERS=1 \
	-DTARGET_PAGE_BITS=12 \
	-DSE_RAM_OBJECT_BITS=12 \
	-D"SE_RAM_OBJECT_MASK=(~11)"
S2E_PATH ?= ../../s2e

Amba.o: src/Amba.cpp inc/Amba.h inc/Numbers.h
	$(CXX) src/Amba.cpp $(CXX-FLAGS) -o Amba.o -c

Test: src/Test.cpp inc/Numbers.h
	$(CXX) src/Test.cpp $(CXX-FLAGS) -o Test

clean:
	-rm *.o
	-rm *.a

../../compile_flags.txt: compile_flags_template.txt
	cat $^ \
	| sed "s|LLVM_PATH|${LLVM_PATH}|" \
	| sed "s|BOOST_PATH|${BOOST_PATH}|" \
	| sed "s|GLIBC_PATH|${GLIBC_PATH}|" \
	| sed "s|ZYDIS_PATH|${ZYDIS_PATH}|" \
	| sed "s|ZYCORE_PATH|${ZYCORE_PATH}|" \
	> $@

compile_flags.txt: ../../compile_flags.txt

.PHONY: clean
