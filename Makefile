CXX ?= g++
AR := gcc-ar

CXXFLAGS ?= -std=c++17 -rdynamic -O3 -flto=jobserver -fno-fat-lto-objects -DENABLE_SDL_GRAPHICS -D_REENTRANT -I/usr/include/SDL2
TOOLFLAGS ?= -I.
EXTRALIBS ?= -lSDL2 -pthread

.PHONY: all clean cloc run run-step run-quiet test test-noninteractive dump

AUTOTESTDIR := test/auto
AUTOTESTSRC := $(AUTOTESTDIR)/prog.kasm

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

test-noninteractive: $(TOOLBINS) $(TESTKASMOBJS)
	bin/run_test_suite -ni

dump: $(TOOLBINS)
	bin/arch_dump

bin/bios.bin: asm/bios.kasm $(TOOLBINS)
	bin/kasm asm/bios.kasm bin/bios.bin

bin/prog.bin: asm/prog.kasm $(TOOLBINS)
	bin/kasm asm/prog.kasm bin/prog.bin

$(AUTOTESTSRC): bin/gen_test_auto
	bin/gen_test_auto

$(KASMOBJS): %.bin: %.kasm ./bin/kasm
	bin/kasm $< $@

$(OBJS): %.o: %.cpp $(HDRS) Makefile
	+$(CXX) $(CXXFLAGS) -c $< -o $@

$(TOOLLIBOBJS): %.o: %.cpp $(HDRS) $(TOOLLIBHDRS) Makefile
	+$(CXX) $(CXXFLAGS) $(TOOLFLAGS) -c -o $@ $<

$(TOOLBINS): bin/%: tools/%.cpp $(HDRS) $(LIB) $(TOOLLIB) Makefile
	+$(CXX) $(CXXFLAGS) $(TOOLFLAGS) -o $@ $< $(TOOLLIB) $(LIB) $(EXTRALIBS)

$(LIB): $(OBJS) Makefile
	mkdir -p bin/lib
	$(AR) rvs $(LIB) $(OBJS)

$(TOOLLIB): $(TOOLLIBOBJS) Makefile
	mkdir -p bin/lib
	$(AR) rvs $(TOOLLIB) $(TOOLLIBOBJS)
