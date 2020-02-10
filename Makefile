.PHONY: all test clean run

SRCS := $(shell find src -type f -name "*.cpp")
OBJS := $(SRCS:.cpp=.o)
HDRS := $(shell find src -type f -name "*.h")
LIB := bin/lib/libkcpu.a

TOOLSRCS := $(shell find tools -maxdepth 1 -type f -name "*.cpp")
TOOLBINS := $(patsubst tools/%.cpp, bin/%, $(TOOLSRCS))

TOOLLIBSRCS := $(shell find tools/lib -type f -name "*.cpp")
TOOLLIBOBJS := $(patsubst %.cpp, %.o, $(TOOLLIBSRCS))
TOOLLIBHDRS := $(shell find tools/lib -type f -name "*.h")
TOOLLIB := bin/lib/libtools.a

KASMSRCS := $(shell find test -type f -name "*.kasm")
KASMOBJS := $(patsubst %.kasm, %.bin, $(KASMSRCS))

CXXFLAGS := -std=c++17 -O3
TOOLFLAGS := -I.

all: $(LIB) $(TOOLLIB) $(KASMOBJS) $(TOOLBINS)
    
test: all
	@./bin/run_test_suite
	
clean:
	rm -f $(OBJS) $(TOOLLIBOBJS) $(KASMOBJS)
	rm -rf bin

bin/bios.bin: asm/bios.kasm
	bin/kasm asm/bios.kasm bin/bios.bin

bin/prog.bin: asm/prog.kasm
	bin/kasm asm/prog.kasm bin/prog.bin

run: all bin/bios.bin bin/prog.bin
	bin/main -v
    
run-quiet: all bin/bios.bin bin/prog.bin
	bin/main

$(KASMOBJS): %.bin: %.kasm ./bin/kasm
	bin/kasm $< $@

$(OBJS): %.o: %.cpp $(HDRS) Makefile
	g++ $(CXXFLAGS) -c $< -o $@

$(TOOLLIBOBJS): %.o: %.cpp $(HDRS) $(TOOLLIBHDRS) Makefile
	g++ $(CXXFLAGS) $(TOOLFLAGS) -c $< -o $@

$(TOOLBINS): bin/%: tools/%.cpp $(HDRS) $(LIB) $(TOOLLIB) Makefile
	g++ $(TOOLFLAGS) $(CXXFLAGS) $< $(TOOLLIB) $(LIB) -o $@
    
$(LIB): $(OBJS) Makefile
	mkdir -p bin/lib
	ar rvs $(LIB) $(OBJS)
    
$(TOOLLIB): $(TOOLLIBOBJS) Makefile
	mkdir -p bin/lib
	ar rvs $(TOOLLIB) $(TOOLLIBOBJS)
