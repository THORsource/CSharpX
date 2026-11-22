# Embedded and target contracts (DRAFT)

Targets supply CPU triple, ABI, linker memory map, startup/reset, vector table, interrupt rules, atomics, MMIO HAL, allocator, panic/exception policy, flash/debug commands and runtime capabilities.

HAL interfaces: GPIO, UART, SPI, I2C, CAN, timers, DMA. Safe public HAL APIs can encapsulate audited unsafe register access. No arbitrary raw MMIO access from safe application code.

`[Interrupt("Timer0")]`, `[NoAllocation]`, `[NoGC]`, `[NoBlocking]`, `[MemoryRegion("FastRAM")]` are proposed attributes. Interrupt analysis checks effects, shared mutable state and synchronization. Static RAM, stacks, heaps and pools must be accounted without double-counting.

Initial candidate targets: ARM Cortex-M, RISC-V, Windows x86-64, Embedded Linux ARM; AURIX/TriCore requires a viable compiler/toolchain backend, not assumed Rust target availability.
