#pragma once

#include <memory>

#include "HeapLeak.h"

namespace data {

class AmbaData {

public:
	AmbaData() :
		m_heap_leak(std::make_unique<heap_leak::HeapLeak>(heap_leak::HeapLeak()))
	{}

	std::unique_ptr<heap_leak::HeapLeak> m_heap_leak;

}; // AmbaData

} // namespace data
