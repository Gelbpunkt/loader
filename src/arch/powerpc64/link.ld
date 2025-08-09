/* https://ftp.rtems.org/pub/rtems/people/sebh/ABI64BitOpenPOWERv1.1_16July2015_pub.pdf */

ENTRY(_start)

phys = 0x400000;

SECTIONS
{
  . = phys;
  loader_start = .;

  .text : {
    *(.text)
    *(.text.*)
  } :segment_rx

  .rodata : ALIGN(16) {
    *(.rodata)
    *(.rodata.*)
  } :segment_ro

  .data   : ALIGN(16) {
    *(.data)
    *(.data.*)
  } :segment_rw

  .data1 : ALIGN(16) {
    *(.data1)
    *(.data1.*)
  } :segment_rw

  TOC_PTR = . + 32K;

  .got    : ALIGN(16) {
    *(.got)
  } :segment_ro

  .toc : ALIGN(16) {
    *(.toc)
    *(.toc.*)
  } :segment_rw

  .sdata : ALIGN(16) {
    *(.sdata)
    *(.sdata.*)
  } :segment_rw

  .sbss : ALIGN(16) {
    *(.sbss)
    *(.sbss.*)
  } :segment_rw

  .plt : ALIGN(16) {
    *(.plt)
    *(.plt.*)
  } :segment_rw

  .bss1   : ALIGN(16) {
    *(.bss)
    *(.bss.*)
  } :segment_rw

  .bss    : ALIGN(16) {
    *(.bss)
    *(.bss.*)
  } :segment_rw

  loader_end = .;
}