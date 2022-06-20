// The design of this module comes directly from Starr Horne's rust-nes.
// See https://github.com/starrhorne/nes-rust/blob/master/src/cartridge/pager.rs
#[derive(Clone, Copy, Debug)]
#[repr(usize)]
pub enum PageSize {
  OneKb = 0x400,
  FourKb = 0x1000,
  EightKb = 0x2000,
  SixteenKb = 0x4000,
}

#[derive(Clone, Copy, Debug)]
pub enum Page {
  First(PageSize),
  Last(PageSize),
  Number(usize, PageSize),
  FromEnd(usize, PageSize),
}

#[derive(Debug)]
pub struct Pager {
  pub data: Vec<u8>,
}

impl Pager {
  pub fn new(data: Vec<u8>) -> Pager {
    Pager { data }
  }

  #[named]
  #[inline]
  pub fn read_u8(&self, page: Page, offset: u16) -> u8 {
    trace_enter!();
    trace_var!(page);
    trace_u16!(offset);
    let index = self.get_index(page, offset);
    trace_u16!(index as u16);
    let result = self.data[index];
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  pub fn write_u8(&mut self, page: Page, offset: u16, value: u8) {
    trace_enter!();
    trace_var!(page);
    trace_u16!(offset);
    trace_u8!(value);
    let index = self.get_index(page, offset);
    trace_u16!(index as u16);
    self.data[index] = value;
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn get_page_count(&self, size: PageSize) -> usize {
    trace_enter!();
    trace_var!(size);
    if self.data.len() % (size as usize) != 0 {
      panic!("Data length must be evenly divisible by page size.");
    }
    let result = self.data.len() / (size as usize);
    trace_var!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  pub fn get_index(&self, page: Page, offset: u16) -> usize {
    use Page::*;
    trace_enter!();
    trace_var!(page);
    trace_u16!(offset);
    let result = match page {
      First(size) => self.get_index(Number(0, size), offset),
      Last(size) => {
        let page_count = self.get_page_count(size);
        trace_var!(page_count);
        let last_page = page_count - 1;
        trace_var!(last_page);
        self.get_index(Number(last_page, size), offset)
      }
      Number(number, size) => {
        let page_count = self.get_page_count(size);
        trace_var!(page_count);
        let last_page = page_count - 1;
        trace_var!(last_page);
        if offset > size as u16 {
          panic!("Offset {} exceeded page bounds {:?}", offset, size);
        }
        if number > last_page {
          panic!("Page {} out of bounds (max: {})", number, last_page);
        }
        number * (size as usize) + offset as usize
      }
      FromEnd(number, size) => {
        let page_count = self.get_page_count(size);
        trace_var!(page_count);
        let last_page = page_count - 1;
        trace_var!(last_page);
        self.get_index(Number(last_page - number, size), offset)
      }
    };
    trace_var!(result);
    trace_exit!();
    result
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;
  use Page::*;
  use PageSize::*;

  fn build_pager() -> Pager {
    let mut data = Vec::new();
    for i in 0..(SixteenKb as usize * 4) {
      data.push(i as u8);
    }
    Pager::new(data)
  }

  #[test]
  fn test_page_count() {
    init();
    let pager = build_pager();
    assert_eq!(4, pager.get_page_count(SixteenKb));
    assert_eq!(8, pager.get_page_count(EightKb));
    assert_eq!(16, pager.get_page_count(FourKb));
  }

  #[test]
  fn test_index_first() {
    init();
    let pager = build_pager();
    assert_eq!(4, pager.get_index(First(SixteenKb), 4));
    assert_eq!(8, pager.get_index(First(SixteenKb), 8));
  }

  #[test]
  fn test_index_last() {
    init();
    let pager = build_pager();
    assert_eq!(0x4000 * 3 + 42, pager.get_index(Last(SixteenKb), 42));
  }

  #[test]
  fn test_index_number() {
    init();
    let pager = build_pager();
    assert_eq!(0x1000 * 3 + 36, pager.get_index(Number(3, FourKb), 36));
  }

  #[test]
  #[should_panic]
  fn test_index_overflow() {
    init();
    let pager = build_pager();
    pager.get_index(First(SixteenKb), SixteenKb as u16 + 1);
  }

  #[test]
  #[should_panic]
  fn test_index_nopage() {
    init();
    let pager = build_pager();
    pager.get_index(Number(100, SixteenKb), 0);
  }

  #[test]
  fn test_rw() {
    init();
    let mut pager = build_pager();
    pager.write_u8(Last(FourKb), 5, 0x66);
    assert_eq!(0x66, pager.read_u8(Last(FourKb), 5));
    assert_eq!(0x66, pager.read_u8(Last(SixteenKb), 0x1000 * 3 + 5));
  }
}
