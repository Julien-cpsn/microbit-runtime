# Microbit runtime

## Bibliography
 - https://azeria-labs.com/writing-arm-assembly-part-1/

## Useful commands

Compile from A&AT ASM

```bash
as assembly.s --32 -o assembly.o
ld -m elf_i386 -dynamic-linker /lib/ld-linux.so.2 -o assembly -lc assembly.o
```

Compile from c to binary

```bash
arm-linux-gnueabihf-gcc -mbe32 -march=armv6 -marm helloworld.c -o helloworld
```

Compile from c to ASM

```bash
arm-linux-gnueabihf-gcc -mbe32 -march=armv6 -marm helloworld.c -S
```

Disassemble program
```bash
arm-linux-gnueabihf-objdump -drwC helloworld
```