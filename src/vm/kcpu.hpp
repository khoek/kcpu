#ifndef VM_KCPU_H
#define VM_KCPU_H

#include <optional>
#include "common.hpp"
#include "mod/ctl.hpp"
#include "mod/reg.hpp"
#include "mod/mem.hpp"
#include "mod/alu.hpp"
#include "mod/io.hpp"

namespace kcpu {

class vm {
    private:
    uint32_t total_clocks;
    vm_logger logger;

    void dump_registers();
    void disassemble_current();

    public:
    enum STATE {
        STATE_RUNNING,
        STATE_HALTED,
        STATE_ABORTED,

        // Not a real state, just returned by run when it times out
        STATE_TIMEOUT,
    };

    mod_ctl ctl;
    mod_reg reg;
    mod_mem mem;
    mod_alu alu;
    mod_io  io;

    vm(vm_logger logger);
    uint32_t get_total_clocks();
    STATE get_state();

    STATE ustep();
    STATE step();
    STATE run(std::optional<uint32_t> max_clocks);
    STATE run();
    void resume();
};

}

#endif
