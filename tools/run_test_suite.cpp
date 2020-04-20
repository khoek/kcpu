#include <filesystem>
#include <algorithm>

#include "lib/binutils.hpp"
#include "src/lang/arch.hpp"
#include "src/vm/kcpu.hpp"

#define PADDING_WIDTH 20
#define MAX_USTEPS 500000000

static std::filesystem::path suite_path("test");

static bool load_binary_try(const char *name, std::filesystem::path file, size_t max_len, void *buff) {
    if(std::filesystem::exists(file)) {
        load_binary(name, file, max_len, buff);
        return true;
    }

    return false;
}

static void load_binary_maybedefault(const char *name, std::filesystem::path p, std::string filename, size_t max_len, void *buff) {
    if(load_binary_try(name, p / filename, max_len, buff)) {
        return;
    }

    if(load_binary_try(name, suite_path / ("default." + filename), max_len, buff)) {
        return;
    }

    throw std::runtime_error("could not find binary or default");
}

static std::string colour_str(std::string s, bool good) {
    return (good ? "\033[1;32m" : "\033[1;31m") + s + "\033[0m";
}

static bool run_test(bool verbose, uint32_t num, const std::filesystem::path path) {
    kcpu::vm_logger logger(verbose);
    kcpu::vm cpu(logger);

    load_binary_maybedefault("BIOS", path, "bios.bin", BIOS_SIZE, cpu.mem.bios.data());
    load_binary_maybedefault("PROG", path, "prog.bin", PROG_SIZE, cpu.mem.prog.data());

    try {
        std::cout << "Test " << std::setw(2) << num << std::left << std::setw(PADDING_WIDTH) << ": '" + path.filename().string() + "' "
                  << std::right << std::setw(0);

        if(verbose) printf("\nCPU Start\n");
        kcpu::vm::state s = cpu.run(MAX_USTEPS);
        if(verbose) printf("\nCPU %s, %ld uinstructions executed taking %ldms\n", cpu.get_state() == kcpu::vm::state::HALTED ? "Halted" : "Aborted", cpu.get_total_clocks(), cpu.get_real_ns_elapsed() / 1000 / 1000);

        switch(s) {
            case kcpu::vm::state::HALTED: {
                std::cout << colour_str("PASS", true)
                          << "  @" << std::setfill(' ') << std::setw(9) << cpu.get_total_clocks()
                          <<  "  " << std::setfill(' ') << std::setw(4) << (cpu.get_real_ns_elapsed() / 1000 / 1000) << "ms"
                          <<  "  (" << std::setfill(' ') << std::setprecision(2) << std::fixed << std::showpoint << std::setw(4) << cpu.get_effective_MHz_freq() << "MhZ)"
                          << std::setw(0) << std::setfill(' ') << std::endl;
                return true;
            }
            case kcpu::vm::state::ABORTED: {
                std::cout << colour_str("FAIL, ABORTED", false) << std::endl;
                return false;
            }
            case kcpu::vm::state::TIMEOUT: {
                std::cout << colour_str("FAIL, DETERMINISTIC TIMEOUT", false) << std::endl;
                return false;
            }
            default: break;
        }
    } catch(kcpu::vm_error &e) {
        std::cout << colour_str("FAIL, EXCEPTION", false) << ": " << e.what() << std::endl;
        return false;
    }

    throw std::runtime_error("abnormal test end condition!");
}

int main(int argc, char **argv) {
    bool headless = true;
    bool noninteractive_mode = false;

    std::vector<std::string> args;
    for(int i = 1; i < argc; i++) {
        std::string arg(argv[i]);
        if(arg == "-ni") {
            noninteractive_mode = true;
        } else if(arg == "-h") {
            headless = true;
        } else if(arg == "-nh") {
            headless = false;
        } else {
            args.push_back(argv[i]);
        }
    }

    if(args.size() > 0) {
        std::cerr << "Unexpected argument: " << args[0] << std::endl;
        return 1;
    }

    graphics::get_graphics().configure(headless);

    std::cout << "--------------------------------------------------------------" << std::endl;

    std::vector<std::filesystem::path> tests;
    for(const auto & entry : std::filesystem::directory_iterator(suite_path)) {
        if(entry.is_directory()) {
            tests.push_back(entry.path());
        }
    }

    std::sort(tests.begin(), tests.end());

    std::vector<std::pair<uint32_t, std::filesystem::path>> failed;
    uint32_t passes = 0;
    for(uint i = 0; i < tests.size(); i++) {
        if(run_test(false, i + 1, tests[i])) {
            passes++;
        } else {
            failed.push_back(std::pair(i, tests[i]));
        }
    }

    std::cout << "--------------------------------------------------------------" << std::endl;
    std::cout << "Test Suite Result: " << colour_str(((tests.size() == passes) ? "SUCCESS" : "FAILED"), tests.size() == passes)
              << ", " << passes << "/" << tests.size() << " passes" << std::endl;

    if(!noninteractive_mode && failed.size()) {
        std::cout << std::endl << "Press any key to re-run " << colour_str("failed", false) << " test '"
                  << failed[0].second.filename().string() << "' verbosely..." << std::endl;

        char c;
        std::cin >> std::noskipws >> c >> std::skipws;
        run_test(true, failed[0].first, failed[0].second);
    }

    return !(tests.size() == passes);
}
