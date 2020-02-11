#ifndef VM_KCPU_H
#define VM_KCPU_H

#include <optional>
#include "common.h"
#include "mod/ctl.h"
#include "mod/reg.h"
#include "mod/mem.h"
#include "mod/alu.h"

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
  
    vm();
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
