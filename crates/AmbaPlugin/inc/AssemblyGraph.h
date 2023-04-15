#pragma once

#include <string>
#include <unordered_map>

#include "ControlFlow.h"

namespace assembly_graph {

using namespace control_flow::types;

Unpacked unpack(Packed packed);

class AssemblyGraph : public control_flow::ControlFlow {
  public:
	AssemblyGraph(std::string);

	amba::TranslationFunction translateBlockStart;
	amba::ExecutionFunction onBlockStart;

  protected:
	StatePC toAlias(UidS2E, u64);
	Packed getBlockId(s2e::S2EExecutionState *, u64);

	/// State uuid → reuses
	std::unordered_map<UidS2E, Packed> m_uuids {};

	/// (State, pc) → gen
	std::unordered_map<StatePC, Generation> m_generations {};

	/// Either:
	/// State → (State, pc)
	/// Alias → Alias
	std::unordered_map<Packed, Packed> m_last {};
};

}
