#include <iostream>
#include <filesystem>

void build(std::istream &in, std::ostream &out);
void build(const char *inpath, const char *outpath);

void load_binary(const char *name, std::filesystem::path path, size_t max_len, void *buff);