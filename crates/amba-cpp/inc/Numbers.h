#pragma once
#include <cstdint>

typedef std::int8_t i8;
typedef std::int16_t i16;
typedef std::int32_t i32;
typedef std::int64_t i64;

typedef std::uint8_t u8;
typedef std::uint16_t u16;
typedef std::uint32_t u32;
typedef std::uint64_t u64;

typedef float f32;
typedef double f64;

typedef __ssize_t isize;
typedef std::size_t  usize;

namespace prefixes {

constexpr usize Ki = 1024;
constexpr usize Mi = 1024 * Ki;
constexpr usize Gi = 1024 * Mi;
constexpr usize Ti = 1024 * Gi;

constexpr usize K = 1'000;
constexpr usize M = 1'000'000;
constexpr usize G = 1'000'000'000;
constexpr usize T = 1'000'000'000'000;

}
