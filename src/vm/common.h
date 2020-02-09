#ifndef COMMON_H
#define COMMON_H

#include <cstdio>

#include "../types.h"
#include "../spec/hw.h"

extern const char * BUS_NAMES[];

#define BUS_DEFAULT_VAL 0x0

class bus_state {
    private:
    bool frozen;
    bool set[NUM_BUSES];
    regval_t bus[NUM_BUSES];

    regval_t get_unset_value(bus_t b);

    public:
    bus_state();
    
    void freeze();
    void assign(bus_t b, regval_t val);
    bool is_assigned(bus_t b);
    void connect(bus_t b1, bus_t b2);
    regval_t early_read(bus_t b);
    regval_t read(bus_t b);
};

#endif
