#pragma once

#include "HeapLeak.h"

namespace data {

struct AmbaData {
	heap_leak::HeapLeak heap_leak;
};

} // namespace data
