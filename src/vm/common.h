#ifndef VM_COMMON_H
#define VM_COMMON_H

#include "../types.h"
#include "../except.h"
#include "../spec/hw.h"

namespace kcpu {

class vm_error : public bt_error {
    public:
    vm_error(const std::string &arg);
};

class vm_logger {
    public:
    bool disassemble;
    bool dump_registers;
    bool dump_bus;

    vm_logger(bool disassemble = false, bool dump_registers = false, bool dump_bus = false);
    
    void logf(const char *fmt, ...);
};

extern const char * BUS_NAMES[];

#define BUS_DEFAULT_VAL 0x0

class bus_state {
    private:
    vm_logger &logger;

    bool frozen;
    bool set[NUM_BUSES];
    regval_t bus[NUM_BUSES];

    regval_t get_unset_value(bus_t b);

    public:
    bus_state(vm_logger &logger);
    
    void freeze();
    void assign(bus_t b, regval_t val);
    bool is_assigned(bus_t b);
    void connect(bus_t b1, bus_t b2);
    regval_t early_read(bus_t b);
    regval_t read(bus_t b);
};

}

#endif
