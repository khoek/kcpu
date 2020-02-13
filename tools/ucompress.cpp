#include <iostream>
#include <unordered_set>
#include <cmath>

#include "src/lang/arch.hpp"
#include "src/spec/hw.hpp"
#include "src/spec/ucode.hpp"

int main() {
    std::unordered_set<uinst_t> all_uis;
    for(uint64_t i = 0; i < INST_MAX; i++) {
        for(uint32_t j = 0; j < UCVAL_MAX; j++) {
            uinst_t ui = kcpu::arch::self().ucode_read(i << INST_SHIFT, j);
            if(ui) {
                all_uis.emplace(ui | (1ULL << 36));
            }
        }
    }

    std::cout << "There are " << UCODE_END << " control bits and " << all_uis.size() << " distinct opcodes." << std::endl;

    // bool implies[UCODE_END][UCODE_END];
    // for(uint32_t i = 0; i < UCODE_END; i++) {
    //     for(uint32_t j = 0; j < UCODE_END; j++) {
    //         implies[i][j] = true;
    //     }
    // }

    // for(uint32_t i = 0; i < UCODE_END; i++) {
    //     for(uint32_t j = 0; j < UCODE_END; j++) {
    //         for(uinst_t ui : all_uis) {
    //             if((ui & (1ULL << i)) && !(ui & (1ULL << j))) {
    //                 implies[i][j] = false;
    //                 break;
    //             }
    //         }
    //     }
    // }

    // for(uint32_t i = 0; i < UCODE_END; i++) {
    //     for(uint32_t j = i + 1; j < UCODE_END; j++) {
    //         if(implies[i][j] && implies[j][i]) {
    //             std::cout << i << " iff " << j << std::endl;
    //         }
    //     }
    // }

    #define SET(ui, b) (!!((ui) & (1ULL << b)))

    std::cout << (log(ACTRL_FLAGS_OUT) / log(2)) << std::endl;
    std::cout << RCTRL_BASE << std::endl;
    std::cout << MCTRL_BASE << std::endl;

    for(uint32_t i = 0; i < UCODE_END; i++) {
        for(uint32_t j = 0; j < UCODE_END; j++) {
            for(uint32_t k = 0; k < UCODE_END;  k++) {
                uint32_t l = 40;
                // for(uint32_t l = 0; l < UCODE_END + 1; l++) {
                    if(i == j || i == k || i == l || j == k || j == l || k == l) continue;

                    bool fail = false;
                    for(uinst_t ui : all_uis) {
                        if(!((SET(ui, i) && !SET(ui, j)) == SET(ui, k))) {
                            fail = true;
                            break;
                        }
                    }
                    if(!fail) {
                        std::cout << i << " && " << j << " -> " << k << std::endl;
                    }
                // }
            }
        }
    }
}

