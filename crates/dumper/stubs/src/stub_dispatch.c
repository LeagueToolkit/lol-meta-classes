#include <common.h>

typedef void *dispatch_block_t;
typedef void *dispatch_data_t;
typedef struct dispatch_queue_s *dispatch_queue_t;

dispatch_data_t $darwin$dispatch_data_create(const void *buffer, size_t size, dispatch_queue_t queue, dispatch_block_t destructor) {
  // NOTE: Should only be used for metal API?
  panic("dispatch_data_create");
  return NULL;
}
