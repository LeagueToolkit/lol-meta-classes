#include <common.h>
#include <math.h>

extern float exp10f (float f);

struct float2 {
  float val[2];
};

float $darwin$__exp10f(float x) {
  // FIXME: is this correct?
  return exp10f(x);
}

struct float2 $darwin$__sincosf_stret(float x) {
  // TODO: is this correct?
  return (struct float2 const){sinf(x), cosf(x)};
}
