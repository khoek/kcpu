MOV $0xAAAA %ra
STW $0 %ra
MOV $0xBBBB %ra
STW $2 %ra
MOV $0xCCCC %ra
STW $4 %ra
MOV $0xDDDD %ra
STW $6 %ra

MOV $0 %ra
MOV $0 %rbp

LD %rbp $0 %ra
CMP $0xAAAA %ra
JNE fail

LD %rbp $2 %ra
CMP $0xBBBB %ra
JNE fail

LD %rbp $4 %ra
CMP $0xCCCC %ra
JNE fail

LD %rbp $6 %ra
CMP $0xDDDD %ra
JNE fail

HLT

fail:
    ABRT