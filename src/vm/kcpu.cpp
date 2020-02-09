#include "kcpu.h"

kcpu::kcpu() : total_clock_cycles(0) {
}

void kcpu::ustep() {
    total_clock_cycles++;

    if(ctl.cbits[CBIT_HALTED]) {
        throw "cpu already halted!";
    }

    regval_t i = ctl.get_inst();
    uinst_t ui = ctl.get_uinst();
    logf("\nIP/UC @ I/UI: 0x%04X/0x%04X @ 0x%04X/0x%04lX\n", ctl.reg[REG_IP], ctl.reg[REG_UC], i, ui);

    if(!ui) {
        throw "executing undefined microcode instruction!";
    }

    bus_state state;

    ctl.clock_outputs(ui, state);
    // `mod_reg` must appear before `mod_mem`
    alu.clock_outputs(ui, state);
    reg.clock_outputs(i, ui, state);
    mem.clock_outputs(ui, state);

    mem.clock_connects(ui, state);

    state.freeze();

    alu.clock_inputs(ui, state);
    mem.clock_inputs(ui, state);
    reg.clock_inputs(i, ui, state);
    ctl.clock_inputs(ui, state);

    ctl.dump_registers();
    mem.dump_registers();
    reg.dump_registers();
    alu.dump_registers();
}

void kcpu::step() {
    do {
        ustep();
    } while(ctl.reg[REG_UC]);
}

uint32_t kcpu::run() {
    while(!ctl.cbits[CBIT_HALTED]) {
        step();
    }

    return total_clock_cycles;
}
