#ifndef VM_MOD_ALU_H
#define VM_MOD_ALU_H

#include "../common.h"

namespace kcpu {

struct op_result {
    uint16_t val;
    uint16_t flags;
};

struct op {
    const char *nm;
    int mode;
    op_result (*eval)(uint16_t a, uint16_t b);
};

class mod_alu {
    private:
    vm_logger &logger;

    op_result result;

    public:
    mod_alu(vm_logger &logger);
    void dump_registers();
    void clock_outputs(uinst_t ui, bus_state &s);
    void clock_inputs(uinst_t ui, bus_state &s);
};

}

#endif