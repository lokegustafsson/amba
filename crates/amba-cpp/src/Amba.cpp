#include "Numbers.h"

#include <s2e/ConfigFile.h>
#include <s2e/S2E.h>
#include <s2e/Utils.h>

#include "Amba.h"

namespace s2e {
namespace plugins {

S2E_DEFINE_PLUGIN(Amba, "Amba S2E plugin", "", );

void Amba::initialize() {
	m_traceBlockTranslation = this
		->s2e()
		->getConfig()
		->getBool(this->getConfigKey() + ".traceBlockTranslation");
	m_traceBlockExecution = this
		->s2e()
		->getConfig()
		->getBool(this->getConfigKey() + ".traceBlockExecution");
	this->s2e()
		->getCorePlugin()
		->onTranslateBlockStart
		.connect(sigc::mem_fun(
			*this,
			&Amba::slotTranslateBlockStart
		));
}

void Amba::slotTranslateBlockStart(
	ExecutionSignal *signal,
	S2EExecutionState *state,
	TranslationBlock *tb,
	uint64_t pc
) {
	if (this->m_traceBlockTranslation) {
		this->getDebugStream(state) << "Translating block at " << hexval(pc) << "\n";
	}
	if (this->m_traceBlockExecution) {
		signal->connect(sigc::mem_fun(*this, &Amba::slotExecuteBlockStart));
	}
}

void Amba::slotExecuteBlockStart(S2EExecutionState *state, u64 pc) {
	getDebugStream(state) << "Executing block at " << hexval(pc) << "\n";
}

} // namespace plugins
} // namespace s2e
