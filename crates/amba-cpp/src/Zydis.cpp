#include <Zydis/Decoder.h>
#include <Zydis/DecoderTypes.h>
#include <Zydis/SharedTypes.h>

#include <cstdio>
#include <exception>
#include <span>
#include <tuple>

#include "Zydis.h"
#include "AmbaException.h"

namespace zydis {

Decoder::Decoder(Arch arch) {
	ZydisMachineMode mode;
	ZydisStackWidth width;

	if (arch == Arch::x86_64) {
		mode = ZYDIS_MACHINE_MODE_LONG_64;
		width = ZYDIS_STACK_WIDTH_64;
	} else {
		AMBA_THROW();
	}

	const auto ret = ZydisDecoderInit(
		&this->m_decoder,
		mode,
		width
	);
	if (ZYAN_FAILED(ret)) {
		AMBA_THROW();
	}
}

std::tuple<ZydisDecodedInstruction, std::vector<ZydisDecodedOperand>>
Decoder::decode(const u8 * const data, const size_t len) const {
	ZydisDecodedInstruction inst;
	std::vector<ZydisDecodedOperand> operands;

	// Grow to fit max operand count elements.
	// Is not a reserve because the shrink would replace operands
	// with default constructed data
	operands.resize(ZYDIS_MAX_OPERAND_COUNT);

	const auto ret = ZydisDecoderDecodeFull(
		&this->m_decoder,
		(void *) data,
		len,
		&inst,
		operands.data()
	);
	if (ZYAN_FAILED(ret)) {
		AMBA_THROW();
	}

	// And shrink back down to the actual amount of operands
	operands.resize(inst.operand_count);

	return std::make_tuple(inst, operands);
}
	
std::tuple<ZydisDecodedInstruction, std::vector<ZydisDecodedOperand>>
Decoder::decode(const std::vector<u8> &program) const {
	return this->decode(program.data(), program.size());
}

std::tuple<ZydisDecodedInstruction, std::vector<ZydisDecodedOperand>>
Decoder::next(const std::vector<u8> &program, size_t *idx) const {
	// Out of bounds before
	if (program.size() <= *idx) {
		AMBA_THROW();
	}

	const auto t = this->decode(program.data() + *idx, program.size() - *idx);
	*idx += std::get<0>(t).length;

	// Set to -1 if out of bounds afterwards
	if (program.size() <= *idx) {
		*idx = (size_t) -1;
	}

	return t;
}
	
}
