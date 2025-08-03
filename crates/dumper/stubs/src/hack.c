#include <common.h>
#include <setjmp.h>
#include <stdio.h>

struct $darwin$objc2_class_ro {
  uint32_t flags;
  uint32_t ivar_base_start;
  uint32_t ivar_base_size;
  uint32_t reserved;
  void *ivar_lyt;
  char *name;
};

struct $darwin$objc2_class {
  struct $darwin$objc2_class *isa;
  struct $darwin$objc2_class *superclass;
  void *cache;
  void *vtable;
  struct $darwin$objc2_class_ro *info;
};

static jmp_buf jmp_hack = {0};
static int has_hack = 0;

void* $darwin$objc_alloc(void* id) { 
    if (id) {
        struct $darwin$objc2_class_ro* info = ((struct $darwin$objc2_class*)id)->info;
        if (has_hack && info && info->name) {
            if (0 == strcmp("OSXAlertBox", info->name)) {
                longjmp(jmp_hack, 1);
            }
        }
    }
    return 0; 
}

void* $darwin$objc_msgSend(void* id, const char* sel) { return 0; }

void run_until_alert(void (*func)()) {
    has_hack = 1;
    if (setjmp(jmp_hack)) {
        has_hack = 0;
    } else {
        func();
        has_hack = 0;
    }
}
