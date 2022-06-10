use super::super::*;

impl CPU {
    #[inline]
    pub fn opcode_inx(&mut self) {
        self.x = self.x.wrapping_add(1);
        self.set_value_flags(self.x);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_inx_e8_adding_data() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
        assert_eq!(cpu.a, 0xc0, "LDA #$C0 should set A to $C0.");
        assert_eq!(cpu.x, 0xc1, "LDA #$C0, TAX, INX should set X to $C1");
        assert!(
            cpu.status & NEGATIVE_FLAG == NEGATIVE_FLAG,
            "LDA #$C0, TAX, INX should set the negative flag."
        );
        assert!(
            cpu.status & CARRY_FLAG == 0,
            "LDA #$C0, TAX, INX should not set the carry flag."
        );
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.x = 0xff;
        cpu.interpret(vec![0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.x, 1);
        assert!(
            cpu.status & NEGATIVE_FLAG == 0,
            "X = $FF, INX, INX should not set the negative flag."
        );
        assert!(
            cpu.status & CARRY_FLAG == 0,
            "X = $FF, INX, INX should not set the carry flag."
        );
    }
}
