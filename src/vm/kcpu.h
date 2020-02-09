#ifndef KCPU_H
#define KCPU_H

#include "common.h"
#include "mod/ctl.h"
#include "mod/reg.h"
#include "mod/mem.h"
#include "mod/alu.h"

class kcpu {
    private:
    uint32_t total_clock_cycles;

    public:
    mod_ctl ctl;
    mod_reg reg;
    mod_mem mem;
    mod_alu alu;
  
    kcpu();
    
    void ustep();
    void step();
    uint32_t run();
};

#endif
