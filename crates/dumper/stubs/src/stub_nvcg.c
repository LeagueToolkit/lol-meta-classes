#include <stdint.h>

#include <common.h>

typedef struct _CGcontext *CGcontext;

typedef enum { CG_BEHAVIOR_3100 = 3000 } CGbehavior;

CGcontext $darwin$cgCreateContext() {
  // FIXME: this should be safe to stub, cg does not seem to be used anymore!
  // panic("not implemented cgCreateContext");
  return (void *)1;
}

void $darwin$cgDestroyContext(CGcontext context) {
  // FIXME: this should be safe to stub, cg does not seem to be used anymore!
  // panic("not implemented cgDestroyContext");
}

void $darwin$cgSetContextBehavior(CGcontext context, CGbehavior behavior) {
  // FIXME: this should be safe to stub, cg does not seem to be used anymore!
  // panic("not implemented cgSetContextBehavior");
}
