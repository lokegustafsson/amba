#pragma once
#include <Zydis/Zydis.h>
#include <Zydis/DecoderTypes.h>

#include <cstdio>
#include <tuple>
#include <vector>
#include <span>

#include "Numbers.h"

namespace zydis {

enum struct Arch {
	x86,
	x86_64,
};

struct Instruction {
	ZydisDecodedInstruction m_inst;
	std::vector<ZydisDecodedOperand> m_ops;

	bool isDeref() const;
	bool isCall() const;
};

class Decoder {
  private:
	ZydisDecoder m_decoder;

	Instruction decode (const u8* const data, const size_t len) const;
  public:
	Decoder(Arch arch = Arch::x86_64);

	Instruction decode(const std::vector<u8> &program) const;

	Instruction decode(const std::span<const u8> program) const;

	Instruction next(const std::vector<u8> &program, size_t *idx) const;

	Instruction next(const std::span<const u8> program, size_t *idx) const;
};
	
}
