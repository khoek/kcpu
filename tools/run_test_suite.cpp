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

static std::filesystem::path suite_path("test");

bool load_binary_try(const char *name, std::filesystem::path file, size_t max_len, void *buff) {
  if(std::filesystem::exists(file)) {
    load_binary(name, file, max_len, buff);
    return true;
  }

  return false;
}

void load_binary_maybedefault(const char *name, std::filesystem::path p, std::string filename, size_t max_len, void *buff) {
  if(load_binary_try(name, p / filename, max_len, buff)) {
    return;
  }

  if(load_binary_try(name, suite_path / ("default." + filename), max_len, buff)) {
    return;
  }

  throw "error: could not find binary or default";
}

bool run_test(bool verbose, uint32_t num, const std::filesystem::directory_entry & entry) {
  kcpu cpu(vm_logger {verbose});
  load_binary_maybedefault("BIOS", entry.path(), "bios.bin", BIOS_SIZE, cpu.mem.bios.raw);
  load_binary_maybedefault("PROG", entry.path(), "prog.bin", PROG_SIZE, cpu.mem.prog.raw);

  if(verbose) printf("CPU Start\n");
  kcpu::STATE s = cpu.run();
  if(verbose) printf("\nCPU %s, %d uinstructions executed\n", cpu.ctl.cbits[CBIT_ABORTED] ? "Aborted" : "Halted", cpu.get_total_clocks());

  switch(s) {
    case kcpu::STATE_HALTED: {
      std::cout << "Test " << num << ": '" << entry.path().filename().string() << "' PASS" << std::endl;
      return true;
    }
    case kcpu::STATE_ABORTED: {
      std::cout << "Test " << num << ": '" << entry.path().filename().string() << "' FAIL" << std::endl;
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
    std::cout << "Test Result: " << ((test_count == passes) ? "SUCCESS" : "FAILED") << ", " << passes << "/" << test_count << " passes" << std::endl;

    return !(test_count == passes);
  } catch(std::string msg) {
      std::cerr << msg << "\n";
  } catch(const char * msg) {
      std::cerr << msg << "\n";
  }

  return 1;
}
