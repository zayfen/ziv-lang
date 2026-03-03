.section .text
.globl _start

main:
    pushq %rbp
    movq %rsp, %rbp
    subq $16, %rsp

    movq $10, -8(%rbp)
    movq $20, -16(%rbp)
    movq -8(%rbp), %rax
    movq %rax, -32(%rbp)
    movq -16(%rbp), %rax
    movq %rax, -40(%rbp)
    movq -32(%rbp), %rax
    addq -40(%rbp), %rax
    movq %rax, -48(%rbp)
    movq -48(%rbp), %rax
    movq %rax, -24(%rbp)
    movq -24(%rbp), %rax
    movq %rax, -64(%rbp)
    movq -64(%rbp), %rax
    imulq $2, %rax
    movq %rax, -72(%rbp)
    movq -72(%rbp), %rax
    movq %rax, -56(%rbp)
    movq -56(%rbp), %rax
    movq %rax, -88(%rbp)
    movq -88(%rbp), %rax
    subq $5, %rax
    movq %rax, -96(%rbp)
    movq -96(%rbp), %rax
    movq %rax, -80(%rbp)
    movq $0, %rax

    leave
    ret

_start:
    call main
    movq %rax, %rdi
    movq $60, %rax
    syscall
