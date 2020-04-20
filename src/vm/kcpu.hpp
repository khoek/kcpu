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

    void disassemble_current();
    void print_debug_info(regval_t i, uinst_t ui, bool pint);

    public:
    enum state {
        RUNNING,
        HALTED,
        ABORTED,

        // Not a real state, just returned by run when it times out
        TIMEOUT,
    };

    mod_ctl ctl;
    mod_reg reg;
    mod_mem mem;
    mod_alu alu;
    mod_io  ioc;

    vm(vm_logger logger);
    void dump_registers();

    uint32_t get_total_clocks();
    state get_state();

    state ustep();
    state step();
    state run(std::optional<uint32_t> max_clocks);
    state run();
    void resume();
};

}

#endif
