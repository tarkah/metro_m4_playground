# print demangled symbols by default
set print asm-demangle on

# detect unhandled exceptions, hard faults and panics
break DefaultHandler
break HardFault
break rust_begin_unwind

# JLink
target extended-remote :2331
monitor flash breakpoints 1
# allow hprints to show up in gdb
monitor semihosting enable
monitor semihosting IOClient 3

monitor reset
load

# start the process but immediately halt the processor
stepi