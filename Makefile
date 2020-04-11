CXX ?= g++

.PHONY: all clean cloc run run-step run-quiet test dump

SRCS := $(shell find src -type f -name "*.cpp")
OBJS := $(SRCS:.cpp=.o)
HDRS := $(shell find src -type f -name "*.hpp")
LIB := bin/lib/libkcpu.a

TOOLSRCS := $(shell find tools -maxdepth 1 -type f -name "*.cpp")
TOOLBINS := $(patsubst tools/%.cpp, bin/%, $(TOOLSRCS))

TOOLLIBSRCS := $(shell find tools/lib -type f -name "*.cpp")
TOOLLIBOBJS := $(patsubst %.cpp, %.o, $(TOOLLIBSRCS))
TOOLLIBHDRS := $(shell find tools/lib -type f -name "*.hpp")
TOOLLIB := bin/lib/libtools.a

TESTKASMSRCS := $(shell find test -type f -name "*.kasm")
TESTKASMOBJS := $(patsubst %.kasm, %.bin, $(TESTKASMSRCS))

SANDBOXKASMSRCS := $(shell find sandbox -type f -name "*.kasm")
SANDBOXKASMOBJS := $(patsubst %.kasm, %.bin, $(SANDBOXKASMSRCS))

KASMOBJS := $(TESTKASMOBJS) $(SANDBOXKASMOBJS)

SANDBOXARGS := sandbox/bios.bin sandbox/prog.bin

CXXFLAGS := -std=c++17 -rdynamic -O3
TOOLFLAGS := -I.

all: $(LIB) $(TOOLLIB) $(KASMOBJS) $(TOOLBINS)

clean:
	rm -f $(OBJS) $(TOOLLIBOBJS) $(SANDBOXKASMOBJS) $(TESTKASMOBJS)
	rm -rf bin

cloc:
	cloc --read-lang-def=.cloc_lang_def.txt src asm test tools

run: $(SANDBOXKASMOBJS) $(TOOLBINS)
	bin/run_vm -d $(SANDBOXARGS)

run-step: $(SANDBOXKASMOBJS) $(TOOLBINS)
	bin/run_vm -s $(SANDBOXARGS)

run-verbose: $(SANDBOXKASMOBJS) $(TOOLBINS)
	bin/run_vm -s -v $(SANDBOXARGS)

run-quiet: $(SANDBOXKASMOBJS) $(TOOLBINS)
	bin/run_vm $(SANDBOXARGS)

test: $(TOOLBINS) $(TESTKASMOBJS)
	bin/run_test_suite

dump: $(TOOLBINS)
	bin/arch_dump

bin/bios.bin: asm/bios.kasm $(TOOLBINS)
	bin/kasm asm/bios.kasm bin/bios.bin

bin/prog.bin: asm/prog.kasm $(TOOLBINS)
	bin/kasm asm/prog.kasm bin/prog.bin

$(KASMOBJS): %.bin: %.kasm ./bin/kasm
	bin/kasm $< $@

$(OBJS): %.o: %.cpp $(HDRS) Makefile
	$(CXX) $(CXXFLAGS) -c $< -o $@

$(TOOLLIBOBJS): %.o: %.cpp $(HDRS) $(TOOLLIBHDRS) Makefile
	$(CXX) $(CXXFLAGS) $(TOOLFLAGS) -c $< -o $@

$(TOOLBINS): bin/%: tools/%.cpp $(HDRS) $(LIB) $(TOOLLIB) Makefile
	$(CXX) $(TOOLFLAGS) $(CXXFLAGS) $< $(TOOLLIB) $(LIB) -o $@

$(LIB): $(OBJS) Makefile
	mkdir -p bin/lib
	ar rvs $(LIB) $(OBJS)

$(TOOLLIB): $(TOOLLIBOBJS) Makefile
	mkdir -p bin/lib
	ar rvs $(TOOLLIB) $(TOOLLIBOBJS)
