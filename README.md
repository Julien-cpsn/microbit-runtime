# Microbit runtime

## Bibliography

### Writing a VM

 - https://azeria-labs.com/writing-arm-assembly-part-1/
 - https://medium.com/@shan1024/writing-a-simple-vm-in-80-lines-of-code-4fe0e949a0d9
 - https://hardwarebee.com/step-by-step-guide-to-microcontroller-programming/

### Microbit official resources

 - https://tech.microbit.org/software/runtime/
 - https://tech.microbit.org/software/hex-format/
 - https://tech.microbit.org/software/spec-universal-hex/
 - https://github.com/lancaster-university/microbit-dal/
 - https://github.com/lancaster-university/microbit-samples

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

better disassemble I guess
```bash
arm-linux-gnueabihf-objdump -D -bbinary -marm helloworld -Mforce-thumb
```
