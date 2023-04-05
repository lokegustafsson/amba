#include "Numbers.h"

extern "C" {
	void rust_init();
	void rust_on_engine_shutdown();
	void rust_on_state_fork(
		u32 old_state_id,
		u32 *new_state_ids_ptr,
		usize new_state_ids_count
	);
	void rust_on_state_merge(u32 base_state_id, u32 other_state_id);
	void rust_on_translate_block(
		u32 current_state_id,
		u64 block_virtual_addr,
		u8 *block_data,
		usize block_len,
		const char *module_path_cstr,
		u64 module_internal_offset
	);
	void rust_on_watched_block_start_execute(
		u32 current_state_id,
		u64 block_virtual_addr
	);
}
