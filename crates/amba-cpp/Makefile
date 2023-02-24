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
LIBS = \
	-I $(BOOST_PATH) \
	-I $(GLIBC_PATH) \
	-I $(LLVM_PATH)
CXX-FLAGS := \
	-I inc \
	-std=c++20 \
	$(S2E_DIRS) \
	$(LIBS) \
	-DBOOST_BIND_GLOBAL_PLACEHOLDERS=1 \
	-DTARGET_PAGE_BITS=12 \
	-DSE_RAM_OBJECT_BITS=12 \
	-D"SE_RAM_OBJECT_MASK=(~11)"
S2E_PATH ?= ../../s2e

Amba.o: src/Amba.cpp inc/Amba.h
	$(CXX) $(CXX-FLAGS) src/Amba.cpp -o Amba.o -c

clean:
	-rm *.o
	-rm *.a

.PHONY: clean
