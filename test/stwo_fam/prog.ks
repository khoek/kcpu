MOV $0 %ra
MOV $0 %rbp

MOV $0xAAAA %ra
ST %rbp $0 %ra
MOV $0xBBBB %ra
ST %rbp $2 %ra
MOV $0xCCCC %ra
ST %rbp $4 %ra
MOV $0xDDDD %ra
ST %rbp $6 %ra

POP %ra
CMP $0xAAAA %ra
JNE fail

POP %ra
CMP $0xBBBB %ra
JNE fail

POP %ra
CMP $0xCCCC %ra
JNE fail

POP %ra
CMP $0xDDDD %ra
JNE fail

HLT

fail:
    ABRT