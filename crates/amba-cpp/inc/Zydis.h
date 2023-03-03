#pragma once
#include <Zydis/Zydis.h>
#include <Zydis/DecoderTypes.h>

#include <cstdio>
#include <tuple>
#include <vector>
#include <span>

#include "Numbers.h"

namespace zydis {

enum class Arch {
	x86,
	x86_64,
};

class Decoder {
  private:
	ZydisDecoder m_decoder;

	std::tuple<ZydisDecodedInstruction, std::vector<ZydisDecodedOperand>>
	decode (const u8* const data, const size_t len) const;
  public:
	Decoder(Arch arch = Arch::x86_64);

	std::tuple<ZydisDecodedInstruction, std::vector<ZydisDecodedOperand>>
	decode(const std::vector<u8> &program) const;

	std::tuple<ZydisDecodedInstruction, std::vector<ZydisDecodedOperand>>
	decode(const std::span<const u8> program) const;
	
	std::tuple<ZydisDecodedInstruction, std::vector<ZydisDecodedOperand>>
	next(const std::vector<u8> &program, size_t *idx) const;

	std::tuple<ZydisDecodedInstruction, std::vector<ZydisDecodedOperand>>
	next(const std::span<const u8> program, size_t *idx) const;
};
	
}
