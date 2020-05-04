CXX ?= g++
AR ?= ar

CXXWARNFLAGS ?=
CXXLTOFLAGS ?= -flto=jobserver -fno-fat-lto-objects
ARLTOFLAGS ?=
SDLFLAGS ?= -DENABLE_SDL_GRAPHICS -D_REENTRANT -I/usr/include/SDL2
SDLLIBS ?= -lSDL2

CXXFLAGS := -std=gnu++2a -rdynamic -O3 \
    -Wall -Wextra -Wno-unused-function -Wno-unused-variable -Wno-unused-parameter -Werror \
    $(CXXWARNFLAGS) $(CXXLTOFLAGS) $(SDLFLAGS)
ARFLAGS := $(ARLTOFLAGS)
TOOLFLAGS := -I.
EXTRALIBS := $(SDLLIBS) -pthread

.PHONY: all clean cloc run run-step run-quiet test test-noninteractive dump

AUTOTESTDIR := test/auto
AUTOTESTSRC := $(AUTOTESTDIR)/prog.ks

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

TESTKASMSRCS := $(shell find test -type f -name "*.ks")
TESTKASMOBJS := $(patsubst %.ks, %.bin, $(TESTKASMSRCS))

SANDBOXKASMSRCS := $(shell find sandbox -type f -name "*.ks")
SANDBOXKASMOBJS := $(patsubst %.ks, %.bin, $(SANDBOXKASMSRCS))

KASMOBJS := $(TESTKASMOBJS) $(SANDBOXKASMOBJS)

SANDBOXARGS := sandbox/bios.bin sandbox/prog.bin

all: $(LIB) $(TOOLLIB) $(KASMOBJS) $(TOOLBINS)

clean:
	rm -f $(OBJS) $(TOOLLIBOBJS) $(SANDBOXKASMOBJS) $(TESTKASMOBJS)
	rm -rf bin

cloc:
	cloc --read-lang-def=.cloc_lang_def.txt .

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

bin/bios.bin: asm/bios.ks $(TOOLBINS)
	bin/kasm asm/bios.ks bin/bios.bin

bin/prog.bin: asm/prog.ks $(TOOLBINS)
	bin/kasm asm/prog.ks bin/prog.bin

$(AUTOTESTSRC): bin/gen_test_auto
	bin/gen_test_auto

$(KASMOBJS): %.bin: %.ks ./bin/kasm
	bin/kasm $< $@

$(OBJS): %.o: %.cpp $(HDRS) Makefile
	+$(CXX) $(CXXFLAGS) -c $< -o $@

$(TOOLLIBOBJS): %.o: %.cpp $(HDRS) $(TOOLLIBHDRS) Makefile
	+$(CXX) $(CXXFLAGS) $(TOOLFLAGS) -c -o $@ $<

$(TOOLBINS): bin/%: tools/%.cpp $(HDRS) $(LIB) $(TOOLLIB) Makefile
	+$(CXX) $(CXXFLAGS) $(TOOLFLAGS) -o $@ $< $(TOOLLIB) $(LIB) $(EXTRALIBS)

$(LIB): $(OBJS) Makefile
	mkdir -p bin/lib
	$(AR) $(ARLTOFLAGS) rvcs $(LIB) $(OBJS)

$(TOOLLIB): $(TOOLLIBOBJS) Makefile
	mkdir -p bin/lib
	$(AR) $(ARFLAGS) rvcs $(TOOLLIB) $(TOOLLIBOBJS)
