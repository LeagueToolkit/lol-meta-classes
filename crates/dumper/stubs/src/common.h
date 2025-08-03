#pragma once
#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>

#define KERN_SUCCESS 0
#define KERN_PROTECTION_FAILURE 2
#define KERN_INVALID_ARGUMENT 4
#define KERN_ABORTED 14
#define KERN_LOCK_OWNED 40
#define KERN_LOCK_OWNED_SELF 41
#define KERN_SEMAPHORE_DESTROYED 42
#define KERN_NOT_SUPPORTED 46
#define KERN_OPERATION_TIMED_OUT 49

#define panic(fmt, ...)                                                          \
  do {                                                                           \
    fprintf(stderr, "%s:%d" fmt, __FILE__, __LINE__ __VA_OPT__(, ) __VA_ARGS__); \
    asm("int $3");                                                               \
    abort();                                                                     \
  } while (0)

#define assert_that(...)                           \
  do {                                             \
    if (!(__VA_ARGS__)) panic("%s", #__VA_ARGS__); \
  } while (0)
