#include <stddef.h>

struct MemCopyRange {
    size_t srcOffset;
    size_t dstOffset;
    size_t len;
};

struct RustVec {
	void* ptr;
	size_t len;
	size_t capacity;
	void (*destructor)(struct RustVec*);
};

void free_rust_vec(struct RustVec* vec);

void generate_memcpy_ranges(struct RustVec* p_output, size_t width, size_t heihgt);
