

PUSH $0xF1A6
CALL func1
POP %ra
CMP $0xF1A6 %ra
JNE fail

PUSH $0xF1A6
CALL func2
POP %ra
CMP $0xF1A6 %ra
JNE fail

PUSH $0xF1A6
CALL func3
POP %ra
CMP $0xF1A6 %ra
JNE fail

HLT



fail:
    ABRT

func1:
    ENTER

    PUSH $0xDEAD
    PUSH $0xBEEF
    POP %ra
    POP %rb

    LEAVE
    RET


func2:
    ENTER

    PUSH $0xDEAD
    PUSH $0xDEAD
    PUSH $0xDEAD
    PUSH $0xDEAD
    PUSH $0xBEEF
    PUSH $0xBEEF
    PUSH $0xBEEF
    PUSH $0xBEEF

    LEAVE
    RET


func3:
    ENTER

    PUSH $0xDEAD
    PUSH $0xDEAD
    PUSH $0xDEAD
    PUSH $0xDEAD
    PUSH $0xBEEF
    PUSH $0xBEEF
    PUSH $0xBEEF
    PUSH $0xBEEF
    POP %ra
    POP %ra
    POP %ra
    POP %ra
    POP %ra
    POP %ra
    POP %ra
    POP %ra

    LEAVE
    RET


