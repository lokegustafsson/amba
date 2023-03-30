/// Metadata struct.
/// Try to keep C-ffi compatible for as long as possible and when no
/// longer possible, create a constructor that is ffi-compatible
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct NodeMetadata {
	pub id: u64,
}
