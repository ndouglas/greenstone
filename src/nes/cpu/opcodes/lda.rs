use super::super::*;

impl CPU {
    #[inline]
    pub fn opcode_lda(&mut self, param: u8) {
        self.program_counter += 1;
        self.a = param;
        self.set_zero_flag(param == 0);
        self.set_negative_flag(param & NEGATIVE_FLAG == NEGATIVE_FLAG);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lda_0xa9_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.a, 0x05);
        assert!(cpu.status & NEGATIVE_FLAG == 0);
        assert!(cpu.status & CARRY_FLAG == 0);
    }

    #[test]
    fn test_lda_0xa9_sets_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x00, 0x00]);
        assert!(
            cpu.status & ZERO_FLAG == ZERO_FLAG,
            "LDA #$00 should set the zero flag."
        );
        cpu.interpret(vec![0xa9, 0x05, 0x00]);
        assert!(
            cpu.status & ZERO_FLAG == 0,
            "LDA #$05 should not set the zero flag."
        );
    }

    #[test]
    fn test_lda_0xa9_sets_negative_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0xFF, 0x00]);
        assert!(
            cpu.status & NEGATIVE_FLAG == NEGATIVE_FLAG,
            "LDA #$FF should set the negative flag."
        );
        cpu.interpret(vec![0xa9, 0x05, 0x00]);
        assert!(
            cpu.status & NEGATIVE_FLAG == 0,
            "LDA #$05 should not set the negative flag."
        );
    }
}
