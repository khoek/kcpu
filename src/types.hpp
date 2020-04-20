#ifndef TYPES_H
#define TYPES_H

#include <cstdint>

typedef uint16_t regval_t;
typedef uint8_t ucval_t; // microcode counter value

typedef uint64_t uinst_t;
#define UINST_FMT "0x%08lX"

static regval_t byte_flip(regval_t v) {
    return ((v & 0x00FF) << 8) | ((v & 0xFF00) >> 8);
}

#endif