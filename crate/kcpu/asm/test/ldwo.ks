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

LDWO %rbp $0 %ra
CMP $0xAAAA %ra
JNE fail

LDWO %rbp $2 %ra
CMP $0xBBBB %ra
JNE fail

LDWO %rbp $4 %ra
CMP $0xCCCC %ra
JNE fail

LDWO %rbp $6 %ra
CMP $0xDDDD %ra
JNE fail

HLT

fail:
    ABRT