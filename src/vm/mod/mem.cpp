#include <cassert>
#include "../../spec/ucode.hpp"
#include "mem.hpp"

namespace kcpu {

mem_bank::mem_bank(uint32_t bytes, bool rom) : rom(rom), raw(std::vector<uint16_t>(bytes >> 1)) { }

void mem_bank::store(regval_t addr, regval_t val) {
    if(addr >= raw.size()) {
        throw vm_error("out of bounds memory store");
    }

    if(rom) {
        throw vm_error("cannot write to ROM!");
    }

    raw[addr >> 1] = val;
}

regval_t mem_bank::load(regval_t addr) {
    if(addr >= raw.size()) {
        throw vm_error("out of bounds memory load");
    }

    return raw[addr >> 1];
}

regval_t * mem_bank::data() {
    return raw.data();
}

mod_mem::mod_mem(vm_logger &logger) : logger(logger), bios(BIOS_SIZE, true), prog(PROG_SIZE, false) {
    prefix[0] = 0;
    prefix[1] = 0;
    fidd_adr = 0;
    fidd_val = 0;
}

#define F_BANK_SELECT (1 << 7)

void mod_mem::dump_registers() {
    logger.logf("LPFX: %04X FPFX: %04X\n", prefix[0], prefix[1]);
    logger.logf("FIDV: %04X FIDA: %04X\n", fidd_val, fidd_adr);
}

mem_bank & mod_mem::get_bank(bool far) {
    return (prefix[far ? 1 : 0] & F_BANK_SELECT) ? prog : bios;
}

#define SHOULD_USE_PREFIX_FAR(ui) (!!(ui & MCTRL_USE_PREFIX_FAR))

void mod_mem::clock_outputs(uinst_t ui, bus_state &s) {
    if(ui & MCTRL_FIDD_STORE) {
        // Note we are just doing "early" address latching,
        // with `fidd_val` to be updated at the normal time in the inputcall.
        fidd_adr = s.early_read(BUS_A);
    }

    if(!(ui & MCTRL_N_FIDD_OUT)) {
        assert(!(ui & MCTRL_FIDD_STORE));
        s.assign(BUS_F, fidd_val);
    }

    if(!(ui & MCTRL_N_MAIN_OUT)) {
        assert(!(ui & MCTRL_MAIN_STORE));
        if(logger.dump_bus) logger.logf("  MB(%d) -> %04X@%04X\n", SHOULD_USE_PREFIX_FAR(ui), fidd_adr, get_bank(SHOULD_USE_PREFIX_FAR(ui)).load(fidd_adr));
        s.assign(BUS_M, get_bank(SHOULD_USE_PREFIX_FAR(ui)).load(fidd_adr));
    }
}

void mod_mem::clock_connects(uinst_t ui, bus_state &s) {
    bool bm_write = ui & MCTRL_BUSMODE_WRITE;
    bool bm_x = ui & MCTRL_BUSMODE_X;
    bool low_bit_set = fidd_adr & 0x1;

    bool connect_m_hi = low_bit_set != bm_write;
    bool should_flip = low_bit_set != bm_x; // means "should flip" during MCTRL_BUSMODE_CONW_BUSB_MAYBEFLIP
    bool connect_b_lo = bm_write != bm_x;

    switch(ui & MASK_MCTRL_BUSMODE) {
        case MCTRL_BUSMODE_NONE: {
            break;
        }
        case MCTRL_BUSMODE_CONW_BUSM: {
            s.connect(BUS_F, BUS_M);
            break;
        }
        case MCTRL_BUSMODE_CONW_BUSB: {
            s.connect(BUS_F, BUS_B);
            break;
        }
        case MCTRL_BUSMODE_CONW_BUSB_MAYBEFLIP: {
            // Note that we do not need the flexibiltiy of s.connect()
            // here, since we only maybeflip during the second step of
            // a byte read, thus putting data onto BUS_B.
            //
            // That is, there is no reason this can't happen due to
            // ucode design, but we don't use it and don't support it
            // right now.
            assert(!s.is_assigned(BUS_B));
            if(!should_flip) {
                s.assign(BUS_B,           s.early_read(BUS_F));
            } else {
                s.assign(BUS_B, byte_flip(s.early_read(BUS_F)));
            }
            break;
        }
        case MCTRL_BUSMODE_CONH: {
            // Similar to the previous, we only use this busmode to *load*
            // the fiddle register, hence our assumptions here are again
            // safe.
            regval_t val_b = s.early_read(BUS_B);
            regval_t val_m = s.early_read(BUS_M);

            regval_t res = 0;

            if(!connect_m_hi) {
                // M_LO_CONNECT
                res |= val_m & 0x00FF;
                if(connect_b_lo) {
                    // B_LO_TO_HI
                    res |= (val_b & 0x00FF) << 8;
                } else {
                    // B_HI_TO_HI
                    res |= (val_b & 0xFF00) << 0;
                }
            } else {
                // M_HI_CONNECT
                res |= val_m & 0xFF00;
                if(connect_b_lo) {
                    // B_LO_TO_LO
                    res |= (val_b & 0x00FF) >> 0;
                } else {
                    // B_HI_TO_LO
                    res |= (val_b & 0xFF00) >> 8;
                }
            }
            s.assign(BUS_F, res);
            break;
        }
        default: throw vm_error("invalid busmode!");
    }
}

void mod_mem::clock_inputs(uinst_t ui, bus_state &s) {
    if(ui & MCTRL_PREFIX_STORE) {
        assert(!(ui & MCTRL_FIDD_STORE));
        assert(!(ui & MCTRL_MAIN_STORE));
        prefix[SHOULD_USE_PREFIX_FAR(ui)] = s.read(BUS_B);
    }

    if(ui & MCTRL_FIDD_STORE) {
        // Note the address latching happens "early" in the outputcall,
        // so we are just left to update the actual value here.
        fidd_val = s.read(BUS_F);
    }

    if(ui & MCTRL_MAIN_STORE) {
        if(logger.dump_bus) logger.logf("  MB(%d) <- %04X@%04X\n", SHOULD_USE_PREFIX_FAR(ui), fidd_adr, s.read(BUS_M));
        get_bank(SHOULD_USE_PREFIX_FAR(ui)).store(fidd_adr, s.read(BUS_M));
    }
}

}