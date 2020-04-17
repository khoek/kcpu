#ifndef VM_MOD_CTL_H
#define VM_MOD_CTL_H

#include "../common.hpp"

namespace kcpu {

#define CBIT_HALTED    0
#define CBIT_ABORTED   1
/*
    When this bit is set, REG_RIR is disconnected from the ucode eeprom address bus,
    and some other instruction code source is used instead.

    (On a clock rising edge:)
    As CBIT_INSTMASK is SET, the state of the PINT external signal line is latched,
    and the value of the low instruction bit passed to the ucode is connected to the
    latched value. All other instruction bits are made zero. As a result, a NOP opcode
    (0x0) is read if there is no PINT, and _DO_INT opcode (0x1) is read if there is a
    PINT.

    While CBIT_INSTMASK is high, we assert AINT with the latched value of PINT.

    CBIT_INSTMASK can only be cleared by JM_EXIT or JM_MAYBEEXIT, so only when a NOP
    is executing.

****************
HARDWARE NOTE: ACTUALLY, THIS HAS CHANGED A BIT, SEE IMPLEMENTATION of `set_instmask_state` (and the places where it is called)
                                                 FOR WHEN THE "Latch value" can be set (it's not really a latch anymore, I think?)
                                                ESPECIALLY NOTE THE CONDITIONAL EXPRESSION IN THAT FUNCTION where the latch val is set
*************


    If instead _DO_INT is executing due to the presence of the mask, then an ordinary
    JM_ENTER will be issued at the completion of the hardware interrupt handling.
    Consequently, we will attempt to set CBIT_INSTMASK when it is already set. When
    this happens, the PINT external signal line will be relatched as normal, and
    there will have been enough time for the AINT issued earlier to have been noted
    by the PIC (so long as the ucode for _DO_INT is longer than 1 instruction,
    which it must be if we want to save RIP and then set its value to something
    else). Thus, PINT will already be low, and no further handling logic is requried.

    This brings us to the instruction loading ucode of the next NOP, and everything
    then works nicely.
*/
#define CBIT_INSTMASK  2
/*
    Interrupt enable.
*/
#define CBIT_IE        3
/*
    This bit is a bit tricky.
    It is set on CLK rising edge whenever IO_READ or IO_WRITE are asserted.
    It is cleared on CLK falling edge whenever IO_DONE is asserted.

    Moreover, it masks the UC increment and UC unlatching, subject to: if IO_DONE is asserted on a CLK
    falling edge (so that CBIT_IO_WAIT should be cleared) simultaneously the its UC unlatching-inhbit
    function does not occur (that is, IO_DONE hard overrides the UC unlatching-inhibit function of this bit,
    and clears the bit at the same time).

    On the other hand, the UC should inc once on a rising edge of CLK at which time IO_WAIT is simultaneously set.
*/
#define CBIT_IO_WAIT   4

#define NUM_CBITS 5

class mod_ctl {
    private:
    vm_logger &logger;

    uinst_t uinst_latch_val = 0;
    bool pint_latch_val = 0;

    void set_instmask_enabled(uinst_t ui, bool state, bool pint);

    public:
    // FIXME it is unfortunate that these need to be public for the run_vm/simulation tools.
    // But it is nice to keep the member functions in this class only representative of
    // actual hardware functions. Somehow resolve this?
    regval_t reg[NUM_SREGS];
    bool cbits[NUM_CBITS];

    mod_ctl(vm_logger &logger);
    void dump_registers();

    regval_t get_inst();
    uinst_t get_uinst();
    bool is_first_uop();
    bool is_aint_active();

    void clock_outputs(uinst_t ui, bus_state &s);
    void clock_inputs(uinst_t ui, bus_state &s, bool pint);
    void offclock_pulse(bool io_done);
};

}

#endif