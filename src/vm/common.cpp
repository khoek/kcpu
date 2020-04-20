#include <cstdio>
#include <cstdarg>
#include <sstream>
#include "../except.hpp"
#include "common.hpp"

namespace kcpu {

vm_error::vm_error(const std::string &msg) : bt_error(msg) { }

vm_logger::vm_logger(bool disassemble, bool dump_registers, bool dump_bus)
    : disassemble(disassemble), dump_registers(dump_registers), dump_bus(dump_bus) { }

void vm_logger::logf(const char *fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    vprintf(fmt, ap);
    va_end(ap);
}

void vm_logger::logf(const std::string &str) {
    logf("%s", str.c_str());
}

const char * BUS_NAMES[] = {
    "BUS_A",
    "BUS_B",
    "BUS_M",
    "BUS_F",
};

bus_state::bus_state(vm_logger &logger) : logger(logger) {
    frozen = false;
    for(uint i = 0; i < NUM_BUSES; i++) {
        set[i] = false;
        bus[i] = 0;
    }
}

regval_t bus_state::get_unset_value(bus_t b) {
    if(b >= BUS_FIRST_FLOATER) {
        throw vm_error("storing floating levels!");
    }

    return BUS_DEFAULT_VAL;
}

void bus_state::freeze() {
    if(frozen) {
        throw vm_error("bus state already frozen!");
    }

    frozen = true;
}

void bus_state::assign(bus_t b, regval_t val) {
    if(logger.dump_bus) logger.logf("  %s <- %X\n", BUS_NAMES[b], val);

    if(frozen) {
        throw vm_error("bus state frozen!");
    }

    if(set[b]) {
        throw vm_error("out bus collision");
    }

    set[b] = true;
    bus[b] = val;
}

bool bus_state::is_assigned(bus_t b) {
    return set[b];
}

void bus_state::connect(bus_t b1, bus_t b2) {
    if(set[b1] && set[b2]) {
        throw vm_error("connect collision!");
    }

    if(!set[b1] && !set[b2]) {
        throw vm_error("IMPLEMENT THIS! (one could have a default)");
    }

    if(set[b1]) {
        assign(b2, early_read(b1));
    } else {
        assign(b1, early_read(b2));
    }
}

regval_t bus_state::early_read(bus_t b) {
    regval_t ret = set[b] ? bus[b] : get_unset_value(b);
    if(logger.dump_bus) logger.logf("  %s -> %X \n", BUS_NAMES[b], ret);
    return ret;
}

regval_t bus_state::read(bus_t b) {
    if(!frozen) {
        throw vm_error("bus state not yet frozen!");
    }

    return early_read(b);
}

}