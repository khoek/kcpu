#include <sstream>
#include "kcpu.hpp"
#include "../spec/inst.hpp"
#include "../codegen/disassembler.hpp"

namespace kcpu {

vm::vm(vm_logger l) : total_clocks(0), logger(l), ctl(logger), reg(logger), mem(logger), alu(logger), ioc(logger) { }

uint32_t vm::get_total_clocks() {
    return total_clocks;
}

vm::state vm::get_state() {
    if(ctl.cbits[CBIT_HALTED]) {
        return ctl.cbits[CBIT_ABORTED] ? ABORTED : HALTED;
    }

    return RUNNING;
}

void vm::dump_registers() {
    logger.logf("---------------------\n");
    ctl.dump_registers();
    mem.dump_registers();
    reg.dump_registers();
    alu.dump_registers();
    ioc.dump_registers();
    logger.logf("\n");
}

void vm::disassemble_current() {
    regval_t ip = ctl.reg[REG_IP] - ((ctl.cbits[CBIT_INSTMASK] ? 0 : 1) * (INST_GET_LOADDATA(ctl.reg[REG_IR]) ? 4 : 2));
    codegen::bound_instruction b = codegen::disassemble_peek(ip, mem.get_bank(false));

    std::stringstream ss;
    ss << "(0x" << std::hex << std::uppercase << ip << ")  " << codegen::pretty_print(b) << std::endl;

    logger.logf("---------------------\n");
    logger.logf(ss.str().c_str());
    if(!logger.dump_registers) {
        dump_registers();
    }
}

void vm::print_debug_info(regval_t i, uinst_t ui, bool pint) {
    if(logger.disassemble && !ctl.cbits[CBIT_INSTMASK]) {
        disassemble_current();
    }

    if(logger.dump_bus) {
        logger.logf("IP/UC @ I/UI: 0x%04X/0x%04X @ 0x%04X/0x%04lX %s\n", ctl.reg[REG_IP], ctl.reg[REG_UC], i, ui, pint ? "(PINT)" : "");
    }

    if(logger.dump_registers) {
        dump_registers();
    }
}

vm::state vm::ustep() {
    total_clocks++;

    if(ctl.cbits[CBIT_HALTED]) {
        throw vm_error("cpu already halted!");
    }

    // This must be called before `ctl.get_uinst()`, as it sets up the value of the uinst latch.
    ctl.offclock_pulse(ioc.is_io_done());
    ioc.offclock_pulse();

    regval_t i = ctl.get_inst();
    uinst_t ui = ctl.get_uinst();
    bool pint = ioc.get_pic().is_pint_active();

    print_debug_info(i, ui, pint);

    if(!ui) {
        throw vm_error("executing undefined microcode instruction!");
    }

    reg.offclock_pulse(i, ctl.is_first_uop());

    bus_state state(logger);

    ctl.clock_outputs(ui, state);
    alu.clock_outputs(ui, state);
    // `mod_reg` must appear after `mod_ctl` and before `mod_mem`
    reg.clock_outputs(ui, state, i);
    mem.clock_outputs(ui, state);
    ioc.clock_outputs(ui, state);

    mem.clock_connects(ui, state);

    state.freeze();

    ioc.clock_inputs(ui, state);
    mem.clock_inputs(ui, state);
    reg.clock_inputs(ui, state, i);
    alu.clock_inputs(ui, state);
    ctl.clock_inputs(ui, state, ioc.get_pic());

    // This is a bit of a hack since the PIC handles aint asynchronously (at least I think that is how it will be implemented)
    ioc.get_pic().handle_aint(ctl.is_aint_active());

    return get_state();
}

vm::state vm::step() {
    if(ctl.cbits[CBIT_HALTED]) {
        throw vm_error("cpu already halted!");
    }

    do {
        ustep();
    } while(ctl.reg[REG_UC] && !ctl.cbits[CBIT_HALTED]);

    return get_state();
}

vm::state vm::run(std::optional<uint32_t> max_clocks) {
    uint32_t then = total_clocks;

    while(!ctl.cbits[CBIT_HALTED]) {
        if(max_clocks && *max_clocks < (total_clocks - then)) {
            return vm::TIMEOUT;
        }

        step();
    }

    return get_state();
}

vm::state vm::run() {
    return run(std::nullopt);
}

void vm::resume() {
    if(get_state() != ABORTED) {
        throw vm_error("cannot resume, cpu not aborted");
    }

    ctl.cbits[CBIT_HALTED ] = false;
    ctl.cbits[CBIT_ABORTED] = false;
}

}