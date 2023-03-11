#pragma once

#include <tuple>

using AmbaException = std::tuple<int, char const*>;

#define AMBA_THROW() \
	{ throw std::make_tuple(__LINE__, __FILE__); }
#define AMBA_ASSERT(_cond) \
	{ if (!(_cond)) { AMBA_THROW(); } }
