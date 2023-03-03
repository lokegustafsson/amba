#include <cstddef>
#include <cstdio>
#include <span>

#include "AmbaException.h"
#include "Numbers.h"
#include "Zydis.h"

// Foo.cpp
const std::vector<u8> program = {
	0x55, 0x48, 0x89 ,0xe5 ,0xb8 ,0x05 ,0x00 ,0x00 ,0x00 ,0x5d ,0xc3 ,0x0f ,
	0x1f ,0x44 ,0x00 ,0x00 ,0x55 ,0x48 ,0x89 ,0xe5 ,0xe8 ,0x00 ,0x00 ,0x00 ,
	0x00 ,0x83 ,0xc0 ,0x02 ,0x5d ,0xc3
};

auto main(const int argc, const char **argv) -> int {
	try {
		const auto decoder = zydis::Decoder();

		size_t idx = 0;
		while (idx != (size_t) -1) {
			auto idx_ = idx;
			auto [inst, operands] = decoder.next(program, &idx);
			std::printf(
				"%ld:\t%s\n",
				idx_,
				ZydisMnemonicGetString(inst.mnemonic)
			);
		}

		return 0;
	}
	catch (AmbaException& e) {
		std::printf("%s:%d\n", std::get<1>(e), std::get<0>(e));
	}
}
