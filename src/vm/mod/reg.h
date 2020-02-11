#ifndef VM_MOD_REG_H
#define VM_MOD_REG_H

#include "../common.h"

class mod_reg {
    private:
    vm_logger &logger;

    void maybe_assign(bus_state &s, uinst_t ui, uint8_t iunum, uint8_t iu, preg_t r);
    void maybe_read(bus_state &s, uinst_t ui, uint8_t iunum, uint8_t iu, preg_t r);

    public:
    //FIXME make private
    regval_t reg[NUM_PREGS];

    mod_reg(vm_logger &logger);
    void dump_registers();
    regval_t get(preg_t r);
    void clock_outputs(regval_t inst, uinst_t ui, bus_state &s);
    void clock_inputs(regval_t inst, uinst_t ui, bus_state &s);
};

#endif