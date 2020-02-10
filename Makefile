.PHONY: all run

SRCS := $(shell find src -type f -name "*.cpp")
OBJS := $(SRCS:.cpp=.o)
HDRS := $(shell find src -type f -name "*.h")
LIB := bin/libkcpu.a

CXXFLAGS := -std=c++17 -O3 -DDEBUG

all: bin/main bin/kasm
    
run: all
	./bin/kasm && ./bin/main

$(OBJS): %.o: %.cpp $(HDRS) Makefile
	g++ $(CXXFLAGS) -c $< -o $@
    
$(LIB): $(OBJS) Makefile
	mkdir -p bin
	ar rvs $(LIB) $(OBJS)
    
bin/main: main.cpp $(LIB) Makefile
	mkdir -p bin
	g++ $(CXXFLAGS) main.cpp $(LIB) -o bin/main
    
bin/kasm: kasm.cpp $(LIB) Makefile
	mkdir -p bin
	g++ $(CXXFLAGS) kasm.cpp $(LIB) -o bin/kasm
