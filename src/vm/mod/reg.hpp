#ifndef VM_MOD_REG_H
#define VM_MOD_REG_H

#include "../common.hpp"

namespace kcpu {

class mod_reg {
    private:
    vm_logger &logger;

    regval_t reg[NUM_PREGS];

    void maybe_assign(bus_state &s, regval_t inst, uinst_t ui, uint8_t iunum, uint8_t iu, preg_t r);
    void maybe_read(bus_state &s, regval_t inst, uinst_t ui, uint8_t iunum, uint8_t iu, preg_t r);

    public:
    mod_reg(vm_logger &logger);
    void dump_registers();
    regval_t get(preg_t r);

    void clock_outputs(uinst_t ui, bus_state &s, regval_t inst);
    void clock_inputs(uinst_t ui, bus_state &s, regval_t inst);
    void offclock_pulse(regval_t inst, bool first_uop);
};

}

#endif