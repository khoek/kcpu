.PHONY: all run

SRCS := $(shell find src -type f -name "*.cpp")
OBJS := $(SRCS:.cpp=.o)
HDRS := $(shell find src -type f -name "*.h")
LIB := libkcpu.a

CXXFLAGS := -g -std=c++17

$(OBJS): %.o: %.cpp $(HDRS)
	g++ $(CXXFLAGS) -c $< -o $@
    
$(LIB): $(OBJS)
	ar rvs $(LIB) $(OBJS)
    
main: main.cpp libkcpu.a
	g++ $(CXXFLAGS) main.cpp $(LIB) -o main
    
kasm: kasm.cpp libkcpu.a
	g++ $(CXXFLAGS) kasm.cpp $(LIB) -o kasm

all: main kasm
    
run: main kasm
	./kasm && ./main
