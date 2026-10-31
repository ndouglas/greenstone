; chase: 8-bit Nate, some scenery, and a pair of paperclips, played back from a table.
;
; make.py writes data.inc (the palettes, the metasprites, and every actor's x, y and sprite frame on every
; frame) and chase.chr (the tiles). Each frame this program turns the table's row into sprites: Nate
; (actor 0) first, so he never flickers, then the clips, starting from a different one each time, so when
; more than eight sprites share a line the NES's flicker is shared among them.

.include "data.inc"

PPUCTRL   = $2000
PPUMASK   = $2001
PPUSTATUS = $2002
OAMADDR   = $2003
PPUSCROLL = $2005
PPUADDR   = $2006
PPUDATA   = $2007
OAMDMA    = $4014

.segment "HEADER"
  .byte "NES", $1A, 2, 1, $00, $00, 0, 0, 0, 0, 0, 0, 0, 0  ; 2 x 16 KB program, 1 x 8 KB tiles, mapper 0

.segment "ZEROPAGE"
row:      .res 2  ; this frame's row of the choreography
meta:     .res 2  ; the metasprite being drawn
frame:    .res 2
nmi_done: .res 1
first:    .res 1  ; the clip drawn first this frame (0 to ACTORS - 2)
k:        .res 1
x_lo:     .res 1
x_hi:     .res 1
foot:     .res 1
top:      .res 1
off:      .res 1

.segment "OAM"
oam: .res 256

.segment "CODE"
reset:
  sei
  cld
  ldx #$40
  stx $4017               ; no APU frame interrupt
  ldx #$FF
  txs
  inx
  stx PPUCTRL
  stx PPUMASK
  stx $4010
  bit PPUSTATUS
@vblank1:
  bit PPUSTATUS
  bpl @vblank1
  lda #0
@clear:
  sta $0000, x
  sta $0100, x
  sta $0300, x
  sta $0400, x
  sta $0500, x
  sta $0600, x
  sta $0700, x
  inx
  bne @clear
  lda #$FF
@hide_all:
  sta oam, x              ; every sprite below the screen until drawn
  inx
  bne @hide_all
@vblank2:
  bit PPUSTATUS
  bpl @vblank2

  lda #$3F                ; the palettes
  sta PPUADDR
  lda #$00
  sta PPUADDR
  ldx #0
@palette:
  lda palettes, x
  sta PPUDATA
  inx
  cpx #32
  bne @palette

  lda #$20                ; the scenery: the first nametable and its attributes, 1 KB from the table
  sta PPUADDR
  lda #$00
  sta PPUADDR
  lda #<nametable
  sta row
  lda #>nametable
  sta row + 1
  ldx #4
  ldy #0
@scenery:
  lda (row), y
  sta PPUDATA
  iny
  bne @scenery
  inc row + 1
  dex
  bne @scenery

  lda #<choreography
  sta row
  lda #>choreography
  sta row + 1
  lda #0
  sta PPUSCROLL
  sta PPUSCROLL
  lda #%10000000          ; NMI on; sprites and background from the first pattern table
  sta PPUCTRL
  lda #%00011110          ; background and sprites, including the leftmost eight pixels
  sta PPUMASK

main:
  jsr draw
  lda #0
  sta nmi_done
@wait:
  lda nmi_done
  beq @wait
  lda frame               ; hold the last frame once the table runs out
  cmp #<(FRAME_COUNT - 1)
  lda frame + 1
  sbc #>(FRAME_COUNT - 1)
  bcs main
  inc frame
  bne @row
  inc frame + 1
@row:
  clc
  lda row
  adc #ACTORS * 4
  sta row
  lda row + 1
  adc #0
  sta row + 1
  inc first
  lda first
  cmp #ACTORS - 1
  bne main
  lda #0
  sta first
  jmp main

; Fill the OAM buffer from this frame's row: each actor's metasprite at its place, tiles off the screen's
; left or right edge left out, unused sprites hidden.
draw:
  ldx #0                  ; x: the next OAM slot
  stx k
@actor:
  lda k                   ; Nate (0) always first; then clip 1 + (k - 1 + first) mod (ACTORS - 1)
  beq @in_range
  clc
  adc first
  cmp #ACTORS
  bcc @in_range
  sbc #ACTORS - 1         ; carry is set here
@in_range:
  asl a
  asl a
  tay
  lda (row), y
  sta x_lo
  iny
  lda (row), y
  sta x_hi
  iny
  lda (row), y
  sta foot
  iny
  lda (row), y
  cmp #$FF
  beq @next_actor
  tay
  lda metas_lo, y
  sta meta
  lda metas_hi, y
  sta meta + 1
  ldy #0
@tile:
  lda (meta), y           ; dy, from the feet up (negative), or $80 at the end
  cmp #$80
  beq @next_actor
  clc
  adc foot
  sec
  sbc #1                  ; OAM holds the line above the sprite's top
  sta top
  iny
  lda (meta), y           ; tile
  sta oam + 1, x
  iny
  lda (meta), y           ; attributes
  sta oam + 2, x
  iny
  lda (meta), y           ; dx
  clc
  adc x_lo
  sta oam + 3, x
  lda x_hi
  adc #0
  sta off                 ; zero only when the tile's left edge is on the screen
  iny
  lda off
  bne @tile
  lda top
  sta oam, x
  inx
  inx
  inx
  inx
  bne @tile
  rts                     ; all 64 sprites in use
@next_actor:
  inc k
  lda k
  cmp #ACTORS
  bne @actor
  lda #$FF                ; hide the rest, up to the end of the buffer
@hide:
  sta oam, x
  inx
  inx
  inx
  inx
  bne @hide
  rts

nmi:
  pha
  lda #0
  sta OAMADDR
  lda #>oam
  sta OAMDMA
  lda #0
  sta PPUSCROLL
  sta PPUSCROLL
  lda #1
  sta nmi_done
  pla
  rti

irq:
  rti

.segment "VECTORS"
  .addr nmi, reset, irq

.segment "CHARS"
  .incbin "chase.chr"
