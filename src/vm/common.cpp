#include <cstdio>
#include <cstdarg>
#include "common.h"

vm_logger::vm_logger() : verbose(false) { }
vm_logger::vm_logger(bool verbose) : verbose(verbose) { }

void vm_logger::logf(const char *fmt, ...) {
    if(!verbose) {
        return;
    }

    va_list ap;
    va_start(ap, fmt);
    vprintf(fmt, ap);
    va_end(ap);
}

const char * BUS_NAMES[] = {
    "BUS_A",
    "BUS_B",
    "BUS_M",
    "BUS_F",
};
    
bus_state::bus_state(vm_logger &logger) : logger(logger) {
    frozen = false;
    for(int i = 0; i < NUM_BUSES; i++) {
        set[i] = false;
        bus[i] = 0;
    }
}

regval_t bus_state::get_unset_value(bus_t b) {
    if(b >= BUS_FIRST_FLOATER) {
        throw "storing floating levels!";
    }

    return BUS_DEFAULT_VAL;
}

void bus_state::freeze() {
    if(frozen) {
        throw "bus state already frozen!";
    }

    frozen = true;
}

void bus_state::assign(bus_t b, regval_t val) {
    if(frozen) {
        throw "bus state frozen!";
    }

    if(set[b]) {
        throw "out bus collision";
    }

    logger.logf("  %s <- %X\n", BUS_NAMES[b], val);

    set[b] = true;
    bus[b] = val;
}

bool bus_state::is_assigned(bus_t b) {
    return set[b];
}

void bus_state::connect(bus_t b1, bus_t b2) {
    if(set[b1] && set[b2]) {
        throw "connect collision!";
    }
    
    if(!set[b1] && !set[b2]) {
        throw "IMPLEMENT THIS! (one could have a default)";
    }
    
    if(set[b1]) {
        assign(b2, early_read(b1));
    } else {
        assign(b1, early_read(b2));
    }
}

regval_t bus_state::early_read(bus_t b) {
    regval_t ret = set[b] ? bus[b] : get_unset_value(b);
    logger.logf("  %s -> %X \n", BUS_NAMES[b], ret);
    return ret;
}

regval_t bus_state::read(bus_t b) {
    if(!frozen) {
        throw "bus state not yet frozen!";
    }

    return early_read(b);
}