MOV $0x1000 %rsp

MOV $20 %ra
MOV $0 %rc

loop:
CALL fun1
JMP loop

fun1:
SUB $1 %ra
JZ end
CALL fun2
RET

fun2:
MOV $0x0001 %rb
AND %ra %rb
JZ fun2_exit
CALL fun3
fun2_exit:
RET

fun3:
ADD $1 %rc
RET

end:
CMP $0xA %rc
JZ win
ABRT

win:
HLT


