#pragma once
#include <tuple>

#include "Amba.h"

using AmbaException = std::tuple<int, char const*>;

#define AMBA_THROW() { \
	*amba::debug_stream() \
		<< "Failed assertion at " \
		<< __FILE__ \
		<< ':' \
		<< __LINE__ \
		<< '\n'; \
	throw std::make_tuple(__LINE__, __FILE__); \
}
#define AMBA_ASSERT(_cond) { \
	if (!(_cond)) { \
		AMBA_THROW(); \
	} \
}
