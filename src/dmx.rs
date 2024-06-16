use std::{
    thread,
    time::{self, Duration},
};

use serialport::SerialPort;

fn send_dmx_packet(port: &mut dyn SerialPort, channels: &[u8]) {
    let start = time::Instant::now();
    port.set_break();
    thread::sleep(Duration::from_micros(110));
    port.clear_break();
    thread::sleep(Duration::from_micros(16));

    port.write_all(&[0x00]);
    port.write_all(&channels);

    // thread::sleep(self.min_b2b.read().unwrap().saturating_sub(start.elapsed()));
}
