# Test: A merger of the tests "primes" and "flag_tui2nmi", which counts the
#       number of TIs (true instructions, not an instruction load or
#       interrupt handle) taken to execute the tests.

LIHP int_handle

MOV $0x0 %ra
ST data.nmi_count.1 %ra
ST data.nmi_count.2 %ra

# Enable jumper TUI2NMI
IOR $0xD0 %ra
OR $0x0001 %ra
IOW $0xD0 %ra

### Instruction counting starts now ###

    CALL run_tests

    # Disable jumper TUI2NMI
    IOR $0xD0 %ra
    AND $0xFFFE %ra
    IOW $0xD0 %ra

### Instruction counting ends now ###

# Read how many instructions were counted
LD data.nmi_count.1 %ra
LD data.nmi_count.2 %rb

# The correct answer is 350482=0x55912 instructions, i.e.
CMP $0x5912 %ra
JNE fail
CMP $0x0005 %rb
JNE fail

HLT

int_handle:
    PUSHA

    # Check that we are in an NMI
    IOR $0x01 %ra
    CMP $0x0001 %ra
    JNE fail

    # Increment NMI count
    LD data.nmi_count.1 %ra
    INC %ra
    ST data.nmi_count.1 %ra

    # Check if the counter just overflowed, and
    # if so update the second counter.
    CMP $0x0 %ra
    JNE int_handle.skip_overflow
    LD data.nmi_count.2 %ra
    INC %ra
    ST data.nmi_count.2 %ra

    # Check if the second counter just overflowed,
    # and if so abort.
    CMP $0x0 %ra
    JE fail

    int_handle.skip_overflow:
        # Issue EOI
        MOV $0x4000 %ra
        IOW $0x01 %ra

        POPA
        IRET

data.nmi_count.1:
    NOP

data.nmi_count.2:
    NOP

fail:
    LD data.nmi_count.1 %ra
    LD data.nmi_count.2 %rb
    ABRT

run_tests:
    # test cases

    PUSH $5
    PUSH $4
    CALL try_case
    ADD $4 %rsp

    PUSH $197
    PUSH $194
    CALL try_case
    ADD $4 %rsp

    PUSH $569
    PUSH $564
    CALL try_case
    ADD $4 %rsp

    PUSH $1009
    PUSH $998
    CALL try_case
    ADD $4 %rsp

    PUSH $1931
    PUSH $1914
    CALL try_case
    ADD $4 %rsp

    RET

try_case:
    ENTER

    LDWO %rsp $4 %ra

    PUSH %ra
    CALL find_next_prime
    ADD $2 %rsp

    LDWO %rsp $6 %rb
    CMP %rb %ra
    JNZ fail

    LEAVE
    RET

find_next_prime:
    ENTER

    LDWO %rsp $4 %rb
    find_next_prime_loop:
        PUSH %rb
        CALL primetest
        POP %rb

        TST %ra
        JNZ find_next_prime_out
        ADD $1 %rb
        JMP find_next_prime_loop
    find_next_prime_out:
        MOV %rb %ra
        LEAVE
        RET

# arg1 = n, arg2 = m, ret = n mod m
modulo:
    ENTER

    LDWO %rsp $6 %ra
    LDWO %rsp $4 %rb

    modulo_loop:
        CMP %ra %rb
        JL modulo_out
        SUB %rb %ra
        JMP modulo_loop

    modulo_out:
        LEAVE
        RET

# arg1 = n, ret = is_prime(n)
primetest:
    ENTER

    LDWO %rsp $4 %rc
    MOV $2 %rd

    primetest_loop:
        CMP %rd %rc
        JE primetest_out

        PUSH %rc
        PUSH %rd
        CALL modulo
        ADD $4 %rsp

        # modulo returns 0 if %rd divides %rc
        TST %ra
        JZ primetest_fail
        ADD $1 %rd

        JMP primetest_loop

    primetest_out:
        MOV $1 %ra
        LEAVE
        RET

    primetest_fail:
        MOV $0 %ra
        LEAVE
        RET