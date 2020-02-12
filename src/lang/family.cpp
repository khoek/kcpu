#include "arch.h"

namespace kcpu {

#define reg_family arch::self().reg_family

// NOTE We could just enforce the "noconst" information from the underlying aliases---
// instead of tagging it again here, but the upshot is that we can define a different
// alias for a const argument if we want, which I think can save a few uops in a few
// places.

static void gen_mem() {
    // FIXME check during registration that there are no potential mapping collisions!

    reg_family(family("LD", {
        family::mapping("LDW" , { param_wreg(), param_wreg_noconst() }),
        family::mapping("LDBL", { param_wreg(), param_breg_lo_noconst() }),
        family::mapping("LDBH", { param_wreg(), param_breg_hi_noconst() }),

        family::mapping("LDWO" , { param_wreg(), param_wreg(), param_wreg_noconst() }),
    }));

    reg_family(family("LDZ", {
        family::mapping("LDBLZ", { param_wreg(), param_breg_lo_noconst() }),
        family::mapping("LDBHZ", { param_wreg(), param_breg_hi_noconst() }),
    }));

    reg_family(family("ST", {
        family::mapping("STW" , { param_wreg(), param_wreg() }),
        family::mapping("STBL", { param_wreg(), param_breg_lo() }),
        family::mapping("STBH", { param_wreg(), param_breg_hi() }),

        family::mapping("STWO" , { param_wreg(), param_wreg(), param_wreg() }),
    }));
}

static void gen_alu() {
    reg_family(family("ADD", {
        family::mapping("ADD2", { param_wreg(), param_wreg_noconst() }),
        family::mapping("ADD3", { param_wreg(), param_wreg(), param_wreg_noconst() }),
    }));

    reg_family(family("ADDNF", {
        family::mapping("ADD2NF", { param_wreg(), param_wreg_noconst() }),
        family::mapping("ADD3NF", { param_wreg(), param_wreg(), param_wreg_noconst() }),
    }));
}

void internal::register_families() {
    gen_mem();
    gen_alu();
}

}