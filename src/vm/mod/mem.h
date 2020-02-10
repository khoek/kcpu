#ifndef VM_MOD_MEM_H
#define VM_MOD_MEM_H

#include "../common.h"

#define BIOS_SIZE (1 << 13)
#define PROG_SIZE (1 << 21) // FIXME what is the actual value?

class mem_bank {
    private:
    uint32_t size;
    bool rom;

    public:
    regval_t *raw;

    mem_bank(uint32_t size, bool rom);
    void store(regval_t addr, regval_t val);
    regval_t load(regval_t addr);
};

class mod_mem {
    private:
    vm_logger &logger;

    regval_t prefix[2];
    regval_t fidd_adr;
    regval_t fidd_val;

    public:
    mem_bank bios;
    mem_bank prog;

    mem_bank & get_bank(bool far);

    mod_mem(vm_logger &logger);
    void dump_registers();
    void clock_outputs(uinst_t ui, bus_state &s);
    void clock_connects(uinst_t ui, bus_state &s);
    void clock_inputs(uinst_t ui, bus_state &s);
};

#endif
