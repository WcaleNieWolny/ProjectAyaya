#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

struct MemCopyRange {
    size_t src_offset;
    size_t dst_offset;
    size_t len;
};

bool fast_yuv_frame_transform(
	int8_t* p_output,
	uint8_t* p_y_arr, 
	uint8_t* p_cb_arr, 
	uint8_t* p_cr_arr,
	uint8_t* p_color_transform_table,
	struct MemCopyRange* p_ranges,
	size_t ranges_len,
	uint64_t width,
	uint64_t height
){
	size_t area = (size_t) (width * height);

	int8_t* tmp_buf = malloc(area * sizeof(int8_t));
	if (tmp_buf == NULL) {
		fprintf(stderr, "[C ERR] malloc returned NULL!\n");
		return false;
	}

	#pragma omp parallel for simd
	for (size_t index = 0; index < area; index++) {
		size_t y = (size_t)p_y_arr[index];
		size_t cb = (size_t)p_cb_arr[index / 4];
		size_t cr = (size_t)p_cr_arr[index / 4];

		size_t offset = (y * 256 * 256) + (cb * 256) + cr;
		int8_t color = (int8_t)p_color_transform_table[offset];

		//size_t output_offset = *(p_fast_lookup_table + index);
		//*(p_output + output_offset) = color;
		*(tmp_buf + index) = color;
	}

	for (size_t i = 0; i < ranges_len; ++i) {
		struct MemCopyRange memCopyRange = *(p_ranges + i);

		memcpy((void*) p_output + memCopyRange.dst_offset, (void*) tmp_buf + memCopyRange.src_offset, memCopyRange.len);
	}

	free((void*)tmp_buf);

	return true;
}
