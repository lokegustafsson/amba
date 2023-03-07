CXX := g++
LIBS := \
	-idirafter $(S2E_PATH)/klee/include \
	-idirafter $(S2E_PATH)/libcoroutine/include \
	-idirafter $(S2E_PATH)/libcpu/include \
	-idirafter $(S2E_PATH)/libfsigc++/include \
	-idirafter $(S2E_PATH)/libq/include \
	-idirafter $(S2E_PATH)/libs2e/include \
	-idirafter $(S2E_PATH)/libs2ecore/include \
	-idirafter $(S2E_PATH)/libs2eplugins/include \
	-idirafter $(S2E_PATH)/libtcg/include \
	-idirafter $(S2E_PATH)/libvmi/include \
	-idirafter $(S2E_PATH)/tools/lib/X8664BitcodeLibrary \
	-idirafter $(BOOST_PATH) \
	-idirafter $(GLIBC_PATH) \
	-idirafter $(LLVM_PATH) \
	-idirafter $(ZYDIS_PATH)/include \
	-idirafter $(ZYCORE_PATH)/include
ARCHIVES := \
	-L $(ZYDIS_PATH)/lib \
	-l Zydis \
	-L $(ZYCORE_PATH)/lib \
	-l Zycore
CXX-FLAGS := \
	-std=c++20 \
	-Wall \
	-Wextra \
	-g3 \
	-O0 \
	-U _FORTIFY_SOURCE \
	-I inc \
	$(LIBS) \
	-DBOOST_BIND_GLOBAL_PLACEHOLDERS=1 \
	-DTARGET_PAGE_BITS=12 \
	-DSE_RAM_OBJECT_BITS=12 \
	-D"SE_RAM_OBJECT_MASK=(~11)"
S2E_PATH ?= ../../s2e

Amba.so: Amba.o Zydis.o HeapLeak.o AmbaPlugin.o
	$(CXX) -shared $^ $(ARCHIVES) -o $@

Amba.o: \
  src/Amba.cpp \
  inc/Amba.h inc/Numbers.h
	$(CXX) src/Amba.cpp $(CXX-FLAGS) -c -o $@

AmbaPlugin.o: \
  src/AmbaPlugin.cpp \
  inc/AmbaPlugin.h inc/Numbers.h
	$(CXX) src/AmbaPlugin.cpp $(CXX-FLAGS) -c -o $@

HeapLeak.o: \
  src/HeapLeak.cpp \
  inc/HeapLeak.h inc/AmbaException.h inc/Numbers.h
	$(CXX) src/HeapLeak.cpp $(CXX-FLAGS) -c -o $@

Zydis.o: \
  src/Zydis.cpp \
  inc/Zydis.h inc/AmbaException.h inc/Numbers.h
	$(CXX) src/Zydis.cpp $(CXX-FLAGS) -c -o $@

Test: \
  src/Test.cpp \
  inc/Numbers.h inc/AmbaException.h \
  Zydis.o Amba.o
	$(CXX) src/Test.cpp Zydis.o $(CXX-FLAGS) -o $@

clean:
	-rm *.o
	-rm *.a
	-rm *.so
	-rm *.out

../../compile_flags.txt: compile_flags_template.txt
	cat $^ \
	| sed "s|LLVM_PATH|${LLVM_PATH}|" \
	| sed "s|BOOST_PATH|${BOOST_PATH}|" \
	| sed "s|GLIBC_PATH|${GLIBC_PATH}|" \
	| sed "s|ZYDIS_PATH|${ZYDIS_PATH}/include|" \
	| sed "s|ZYCORE_PATH|${ZYCORE_PATH}/include|" \
	> $@

compile_flags.txt: ../../compile_flags.txt

.PHONY: clean
