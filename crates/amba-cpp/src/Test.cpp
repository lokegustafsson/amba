#include <cstdio>
#include <Zydis/Zydis.h>

#include "Numbers.h"
#include "Zydis/Decoder.h"
#include "Zydis/SharedTypes.h"

const u8 program[] = {
	0x55, 0x48, 0x89 ,0xe5 ,0xb8 ,0x05 ,0x00 ,0x00 ,0x00 ,0x5d ,0xc3 ,0x0f ,
	0x1f ,0x44 ,0x00 ,0x00 ,0x55 ,0x48 ,0x89 ,0xe5 ,0xe8 ,0x00 ,0x00 ,0x00 ,
	0x00 ,0x83 ,0xc0 ,0x02 ,0x5d ,0xc3
};

int main(const int argc, const char **argv) {
	ZydisDecodedInstruction inst;
	u64 runtime_address = 0x007FFFFFFF400000;

	ZydisDecoder decoder;
	ZydisDecoderInit(&decoder, ZYDIS_MACHINE_MODE_LONG_64, );

	// Loop over the instructions in our buffer.
	usize offset = 0;
	ZydisDisassembledInstruction instruction;
	while (ZYAN_SUCCESS(ZydisDisassembleIntel(
		/* machine_mode:    */ ZYDIS_MACHINE_MODE_LONG_64,
		/* runtime_address: */ runtime_address,
		/* buffer:          */ program + offset,
		/* length:          */ sizeof(program) - offset,
		/* instruction:     */ &instruction
	))) {
		std::printf("%p  %s\n", runtime_address, instruction.text);
		offset += instruction.info.length;
		runtime_address += instruction.info.length;
	}
}
