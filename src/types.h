#ifndef TYPES_H
#define TYPES_H

#include <cstdint>

typedef uint16_t regval_t;
typedef uint8_t ucval_t; // microcode counter value
typedef uint64_t uinst_t; //FIXME is 32 enough? what will the final size be---insert a check for fitting in reg()

static regval_t byte_flip(regval_t v) {
    return ((v & 0x00FF) << 8) | ((v & 0xFF00) >> 8);
}

#define DEBUG

#ifdef DEBUG
#define logf printf
#else
static void logf(const char *fmt, ...) { }
#endif

#endif