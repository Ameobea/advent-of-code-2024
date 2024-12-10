// matches the implementation used by the ferris_elf benchmark bot:
// https://github.com/indiv0/ferris-elf/commit/342f50639550f0ed463e9261aaa9fffb2fbb9bf0
pub fn leak_to_page_aligned(s: &[u8]) -> &'static [u8] {
  if s.is_empty() {
    #[repr(align(16384))]
    struct Aligned([u8; 0]);
    return const { &Aligned([]).0 };
  }
  let layout = std::alloc::Layout::for_value(s)
    .align_to(16 * 1024)
    .expect("can't align layout");
  // SAFETY: We checked that s is not empty, thus the layout has size > 0
  let ptr = unsafe { std::alloc::alloc(layout) };
  if ptr.is_null() {
    std::alloc::handle_alloc_error(layout);
  }
  // SAFETY: The pointer is not null and is valid for at least the layout of s
  unsafe { std::ptr::copy_nonoverlapping(s.as_ptr(), ptr, s.len()) };
  // SAFETY: The pointer is not null and is valid for at least the layout of s
  unsafe { std::slice::from_raw_parts(ptr, s.len()) }
}
