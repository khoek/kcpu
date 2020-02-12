#include "arch.h"

namespace kcpu {

#define reg_family arch::self().reg_family

static void gen_mem() {
    
}

void internal::register_families() {
    gen_mem();
}

}