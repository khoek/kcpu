#ifndef SPEC_OPCLASS_H
#define SPEC_OPCLASS_H

#include "hw.h"

namespace kcpu {

/* An `opclass` represents one of:
    1. (NO_IU3) a single opcode which ignores IU3.
    2. (IU3_SINGLE) a single opcode which cares about IU3.
    3. (IU3_ALL) a range of opcodes for which IU3 represents a third argument.

   In principle more complicated patterns can be supported.
*/
class opclass {
    public:
    enum kind {
        NO_IU3,
        IU3_SINGLE,
        IU3_ALL,
    };

    regval_t raw;
    kind cls;
    preg_t iu3;

    opclass(regval_t raw);
    opclass(regval_t raw, kind k, preg_t iu3);

    regval_t resolve();
    regval_t resolve(preg_t r);
    regval_t resolve_dummy();

    opclass add_flag(regval_t flag);
};

opclass opclass_iu3_single(regval_t raw, preg_t iu3);
opclass opclass_iu3_all(regval_t raw);

}

#endif