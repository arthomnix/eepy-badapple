#!/bin/sh

rm -rf out
mkdir out
mkdir out/bins

for slot in `seq -f "%02g" 31`
do
  EPD_PROG_SLOT=$slot cargo build $@
  elf2epb -i target/thumbv6m-none-eabi/release/eepy-badapple -o "out/bins/badapple.s$slot.epb"
done

(cd out/bins && tar --zstd -cf ../badapple.epa *)