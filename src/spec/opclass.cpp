#include <sstream>
#include <cassert>

#include "opclass.h"
#include "inst.h"

namespace kcpu {

opclass::opclass(regval_t raw, kind cls, preg_t iu3) : raw(raw), cls(cls), iu3(iu3) {
    if(cls != opclass::IU3_SINGLE) {
        assert(!iu3);
    }

    if(cls == opclass::IU3_ALL) {
        assert(raw == INST_STRIP_IU3(raw));
    }

    if(raw & P_I_LOADDATA) {
        std::stringstream ss;
        ss << "opclass raw " << raw << " has LOADDATA bit set!";
        throw ss.str();
    }
}

opclass::opclass(regval_t raw) : opclass(raw, opclass::NO_IU3, (preg_t) 0) { }

regval_t opclass::resolve() {
    switch(cls) {
        case opclass::NO_IU3: {
            return raw;
        }
        case opclass::IU3_SINGLE: {
            return raw | iu3;
        }
        case opclass::IU3_ALL: {
            throw "cannot resolve class";
        }
        default: throw "unknown opclass";
    }
}

opclass opclass_iu3_single(regval_t raw, preg_t iu3) {
    return opclass(raw, opclass::IU3_SINGLE, iu3);
}

opclass opclass_iu3_all(regval_t raw) {
    return opclass(raw, opclass::IU3_ALL, (preg_t) 0);
}

regval_t opclass::resolve(preg_t r) {
    switch(cls) {
        case opclass::NO_IU3: {
            throw "cannot resolve class";
        }
        case opclass::IU3_SINGLE: {
            throw "cannot resolve class";
        }
        case opclass::IU3_ALL: {
            return raw | r;
        }
        default: throw "unknown opclass";
    }
}

regval_t opclass::resolve_dummy() {
    switch(cls) {
        case opclass::NO_IU3:
        case opclass::IU3_SINGLE: {
            return resolve();
        }
        case opclass::IU3_ALL: {
            return resolve((preg_t) 0);
        }
        default: throw "unknown opclass";
    }
}

opclass opclass::add_flag(regval_t flag) {
    return opclass(raw | flag, cls, iu3);
}

}