#include <cstdlib>
#include <iostream>
#include <cstring>
#include <fstream>
#include <filesystem>
#include <execinfo.h>
#include <signal.h>
#include <unistd.h>

#include "lib/compiler.h"
#include "src/gen/arch.h"
#include "src/vm/kcpu.h"

#define PADDING_WIDTH 17
#define MAX_USTEPS 5000000

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

  throw "error: could not find binary or default";
}

static std::string colour_str(std::string s, bool good) {
  return (good ? "\033[1;32m" : "\033[1;31m") + s + "\033[0m";
}

static bool run_test(bool verbose, uint32_t num, const std::filesystem::path path) {
  kcpu cpu(vm_logger {verbose});
  load_binary_maybedefault("BIOS", path, "bios.bin", BIOS_SIZE, cpu.mem.bios.raw);
  load_binary_maybedefault("PROG", path, "prog.bin", PROG_SIZE, cpu.mem.prog.raw);

  if(verbose) printf("CPU Start\n");
  kcpu::STATE s = cpu.run(MAX_USTEPS);
  if(verbose) printf("\nCPU %s, %d uinstructions executed\n", cpu.ctl.cbits[CBIT_ABORTED] ? "Aborted" : "Halted", cpu.get_total_clocks());

  std::cout << "Test " << std::setw(2) << num << std::left << std::setw(PADDING_WIDTH) << ": '" + path.filename().string() + "' "
            << std::right << std::setw(0);

  switch(s) {
    case kcpu::STATE_HALTED: {
      std::cout << colour_str("PASS", true) << "  @" << std::setfill(' ') << std::setw(8) << cpu.get_total_clocks()
                << std::setw(0) << std::setfill(' ') << std::endl;
      return true;
    }
    case kcpu::STATE_ABORTED: {
      std::cout << colour_str("FAIL, ABORTED", false) << std::endl;
      return false;
    }
    case kcpu::STATE_TIMEOUT: {
      std::cout << colour_str("FAIL, DETERMINISTIC TIMEOUT", false) << std::endl;
      return false;
    }
  }

  throw "abnormal test end condition!";
}

int main() {
  try {
    init_arch();

    std::cout << "--------------------------------------------" << std::endl;

    std::vector<std::pair<uint32_t, std::filesystem::path>> failed;
    uint32_t test_count = 0;
    uint32_t passes = 0;
    for (const auto & entry : std::filesystem::directory_iterator(suite_path)) {
      if(entry.is_directory()) {
        test_count++;
        if(run_test(false, test_count, entry)) {
          passes++;
        } else {
          failed.push_back(std::pair(test_count, entry.path()));
        }
      }
    }

    std::cout << "--------------------------------------------" << std::endl;
    std::cout << "Test Suite Result: " << colour_str(((test_count == passes) ? "SUCCESS" : "FAILED"), test_count == passes)
              << ", " << passes << "/" << test_count << " passes" << std::endl;

    if(failed.size()) {
      std::cout << std::endl << "Press any key to re-run " << colour_str("failed", false) << " test '"
                << failed[0].second.filename().string() << "' verbosely..." << std::endl;

      char c;
      std::cin >> std::noskipws >> c >> std::skipws;
      run_test(true, failed[0].first, failed[0].second);
    }

    return !(test_count == passes);
  } catch(std::string msg) {
      std::cerr << msg << "\n";
  } catch(const char * msg) {
      std::cerr << msg << "\n";
  }

  return 1;
}
