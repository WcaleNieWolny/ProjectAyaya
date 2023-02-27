#include <stddef.h>

struct MemCopyRange {
    size_t srcOffset;
    size_t dstOffset;
    size_t len;
};

struct MemCopyRangeOutput {
	struct MemCopyRange* p_mem_ranges;
	size_t mem_ranges_len;
};

 void generate_memcpy_ranges(struct MemCopyRangeOutput* p_output, size_t width, size_t heihgt);
