#pragma once

#include <functional>

// https://stackoverflow.com/questions/17016175/c-unordered-map-using-a-custom-class-type-as-the-key

namespace hashable_wrapper {

/// `std::unordered_map` requires a hash function and an equality
/// operator.  This is a wrapper class that takes a hashable type and
/// creates a new hashable type.
/// Equivalent to ```rust
/// #[derive(Eq, Hash, Default)]
/// struct HashableWrapper<T: Eq + Hash + Default>(T);
/// ```
template <typename T, int I> struct HashableWrapper {
	T val;

	HashableWrapper()
		: val(T {})
	{}

	HashableWrapper(T t)
		: val(t)
	{}

	bool operator==(const HashableWrapper<T, I> &other) const {
		return this->val == other.val;
	}
};

}

template <typename T, int I> struct std::hash<hashable_wrapper::HashableWrapper<T, I>> {
	std::size_t operator()(const hashable_wrapper::HashableWrapper<T, I>& k) const {
		return hash<T>()(k.val);
	}
};
