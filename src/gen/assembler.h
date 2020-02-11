#ifndef GEN_ASSEMBLER_H
#define GEN_ASSEMBLER_H

#include <vector>
#include <istream>
#include "../types.h"
#include "../except.h"

namespace kcpu {

namespace assembler {
    class parse_error : public bt_error {
        public:
        parse_error(uint32_t line, const std::string &arg);
    };

    class internal_error : public bt_error {
        public:
        internal_error(uint32_t line, const std::string &arg);
    };
};

std::vector<regval_t> assemble(std::istream *in);

}

#endif