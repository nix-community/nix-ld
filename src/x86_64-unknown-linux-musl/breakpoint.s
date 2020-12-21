.intel_syntax noprefix
.global _breakpoint
_breakpoint:
    int3
    ret
