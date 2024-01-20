.align 3

.section .text
.globl _start

_start:
    addi a0, zero, -4
    addi a1, zero, -5
    slti a2, a1, -4
    slli a2, a0, 60
    srli a3, a2, 1
    srai a4, a1, 1
    sltiu a5, a1, -4
    andi a6, a0, 4
    ori a7, a0, 4
    xori s2, a0, -1
    auipc s3, 0xdead
    lui s4, 0xdead
    addi a0, a0, -1
