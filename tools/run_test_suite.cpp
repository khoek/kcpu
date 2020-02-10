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
#define MAX_USTEPS 1000000

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

static bool run_test(bool verbose, uint32_t num, const std::filesystem::directory_entry & entry) {
  kcpu cpu(vm_logger {verbose});
  load_binary_maybedefault("BIOS", entry.path(), "bios.bin", BIOS_SIZE, cpu.mem.bios.raw);
  load_binary_maybedefault("PROG", entry.path(), "prog.bin", PROG_SIZE, cpu.mem.prog.raw);

  if(verbose) printf("CPU Start\n");
  kcpu::STATE s = cpu.run(MAX_USTEPS);
  if(verbose) printf("\nCPU %s, %d uinstructions executed\n", cpu.ctl.cbits[CBIT_ABORTED] ? "Aborted" : "Halted", cpu.get_total_clocks());

  std::cout << "Test " << num << std::left << std::setw(PADDING_WIDTH) << ": '" + entry.path().filename().string() + "' ";

  switch(s) {
    case kcpu::STATE_HALTED: {
      std::cout << colour_str("PASS", true) << std::endl;
      return true;
    }
    case kcpu::STATE_ABORTED: {
      std::cout << colour_str("FAIL, ABORTED", false) << std::endl;
      return false;
    }
    case kcpu::STATE_TIMEOUT: {
      std::cout << colour_str("FAIL, TIMEOUT", false) << std::endl;
      return false;
    }
    default: {
      throw "abnormal test end condition!";
    }
  }

}

int main() {
  bool verbose = false;

  try {
    init_arch();

    std::cout << "--------------------------------------------" << std::endl;

    uint32_t test_count = 0;
    uint32_t passes = 0;
    for (const auto & entry : std::filesystem::directory_iterator(suite_path)) {
      if(entry.is_directory()) {
        test_count++;
        passes += run_test(verbose, test_count, entry) ? 1 : 0;
      }
    }

    std::cout << "--------------------------------------------" << std::endl;
    std::cout << "Test Result: " << colour_str(((test_count == passes) ? "SUCCESS" : "FAILED"), test_count == passes)
              << ", " << passes << "/" << test_count << " passes" << std::endl;

    return !(test_count == passes);
  } catch(std::string msg) {
      std::cerr << msg << "\n";
  } catch(const char * msg) {
      std::cerr << msg << "\n";
  }

  return 1;
}
