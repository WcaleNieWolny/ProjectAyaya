#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <pthread.h>

#include "data_structures.h"
#include "logger.h"

CircularBuffer* circular_buffer_init(size_t size, size_t item_size) {
	CircularBuffer* p_buffer = calloc(1, sizeof(CircularBuffer));

	if (p_buffer == NULL) {
		log_error("Cannot malloc circular_buffer");
		return NULL;
	}

	void* p_data_buffer = malloc(size * item_size);
	if (p_data_buffer == NULL) {
		free(p_buffer);
		log_error("Cannot malloc circular_buffer data_buffer");
		return NULL;
	}

	if (pthread_spin_init(&p_buffer->lock, 0) != 0) {
		free(p_buffer);
		free(p_data_buffer);
		log_error("Cannot pthread_spin_init");
		return NULL;
	}

	p_buffer->buffer = p_data_buffer; 
	p_buffer->capacity = size;
	p_buffer->item_size = item_size;

	return p_buffer;
};

bool circular_buffer_lock(CircularBuffer* p_buffer) {
	if (pthread_spin_lock(&p_buffer->lock) == 0) {
		return true;
	}else {
		log_error("Cannot lock spinlock on circular_buffer");
		return false;
	}
}

bool circular_buffer_unlock(CircularBuffer* p_buffer) {
	if (pthread_spin_unlock(&p_buffer->lock) == 0) {
		return true;
	}else {
		log_error("Cannot unlock spinlock on circular_buffer");
		return false;
	}
};

void circular_buffer_free(CircularBuffer* p_buffer) {
	free(p_buffer->buffer);
	pthread_spin_destroy(&p_buffer->lock);
	free(p_buffer);
}


void* circular_buffer_write(CircularBuffer* p_buffer) {
	if (p_buffer->len == p_buffer->capacity) {
		return NULL;
	};

	void* start_ptr = ((void*) (((uint8_t*) p_buffer->buffer) + (p_buffer->item_size * p_buffer->write_i)));
	memset(start_ptr, 0, p_buffer->item_size);

	p_buffer->len += 1;
	p_buffer->write_i += 1;

	if (p_buffer->write_i == p_buffer->capacity) {
		p_buffer->write_i = 0;
	}

	return start_ptr;
}

void* circular_buffer_read(CircularBuffer* p_buffer) {
	if (p_buffer->len == 0) {
		return NULL;
	}

	void* start_ptr = ((void*) (((uint8_t*) p_buffer->buffer) + (p_buffer->item_size * p_buffer->read_i)));
	
	p_buffer->read_i += 1;
	p_buffer->len -= 1;

	if (p_buffer->read_i == p_buffer->capacity) {
		p_buffer->read_i = 0;
	}

	return start_ptr;
}
