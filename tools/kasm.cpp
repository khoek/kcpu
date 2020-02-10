#include <iostream>
#include <sstream>
#include <fstream>
#include <fstream>

#include "lib/compiler.h"
#include "src/gen/arch.h"

int main(int argc, char **argv) {
    if(argc != 3) {
        std::cerr << "Need two arguments" << std::endl;
        return 1;
    }

    try {
        init_arch();
        build(argv[1], argv[2]);

        return 0;
    } catch(std::string msg) {
        std::cerr << msg << '\n';
    } catch(const char * msg) {
        std::cerr << msg << "\n";
        
    }

    return 1;
}