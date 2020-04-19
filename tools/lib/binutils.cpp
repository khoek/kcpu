#include <fstream>
#include "binutils.hpp"

#include "src/codegen/assembler.hpp"

void build(std::istream &in, std::ostream &out) {
    auto ops = kcpu::codegen::assemble(&in);
    out.write((const char *) ops.data(), ops.size() * (sizeof(regval_t) / sizeof(uint8_t)));
}

void build(const char *inpath, const char *outpath) {
    std::ifstream in(inpath);
    std::ofstream out(outpath, std::ios::binary);
    build(in, out);
}

void load_binary(const char *name, std::filesystem::path path, size_t max_len, void *buff) {
    std::ifstream f(path);

    f.seekg(0, std::ios::end);
    size_t len = f.tellg();
    f.seekg(0, std::ios::beg);

    if(len > max_len) {
        printf("%s binary too long!", name);
    }

    f.read((char *) buff, len);
    f.close();
}