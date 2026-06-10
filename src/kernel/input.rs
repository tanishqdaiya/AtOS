use crate::kernel::peripherals::Uart;

pub fn getch() -> u8 {
    let uart = Uart;
    let c = uart.read_byte();
    if c == b'\r' {
	b'\n'
    } else {
	c
    }
}

pub fn getline(buf: &mut [u8]) -> usize {
    let uart = Uart;
    let mut r = 0;

    while r < buf.len() {
	let c = getch();
	uart.write_byte(c);

	buf[r] = c;
	r += 1;

	if c == b'\n' {
	    break;
	}
    }

    r
}

