#ifndef CODEGEN_ASSEMBLER_H
#define CODEGEN_ASSEMBLER_H

#include <vector>
#include <istream>
#include "../types.hpp"
#include "../except.hpp"

namespace kcpu {

namespace codegen {

    class parse_error : public bt_error {
        public:
        parse_error(uint32_t line, const std::string &arg);
    };

    class internal_error : public bt_error {
        public:
        internal_error(uint32_t line, const std::string &arg);
    };

std::vector<regval_t> assemble(std::istream *in);

}

}

#endif