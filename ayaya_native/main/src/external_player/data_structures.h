#include <pthread.h>
#include <stdint.h>
#include <stdbool.h>

typedef struct {
	pthread_spinlock_t lock;
	size_t len;
	size_t capacity;
	size_t write_i;
	size_t read_i;
	size_t item_size;
	void* buffer;
} CircularBuffer;

CircularBuffer* circular_buffer_init(size_t size, size_t item_size);
bool circular_buffer_lock(CircularBuffer* p_buffer);
bool circular_buffer_unlock(CircularBuffer* p_buffer);
void circular_buffer_free(CircularBuffer* p_buffer);
void* circular_buffer_write(CircularBuffer* p_buffer);
void* circular_buffer_read(CircularBuffer* p_buffer);
