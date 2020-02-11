#ifndef GEN_ASSEMBLER_H
#define GEN_ASSEMBLER_H

#include <vector>
#include <istream>
#include "../types.h"

namespace kcpu {

std::vector<regval_t> assemble(std::istream *in);

}

#endif