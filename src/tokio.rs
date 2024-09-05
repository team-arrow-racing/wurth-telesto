//! Implement `embedded-io-async` for `tokio-serial`

use std::io::{Read, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_serial::SerialStream;

/// Creates a shared stream from a [`SerialStream`]
pub fn split_stream(stream: SerialStream) -> (SerialTx, SerialRx) {
    let serial = Arc::new(Mutex::new(stream));

    (
        SerialTx {
            serial: serial.clone(),
        },
        SerialRx {
            serial: serial.clone(),
        },
    )
}

pub struct SerialTx {
    serial: Arc<Mutex<SerialStream>>,
}

impl embedded_io_async::ErrorType for SerialTx {
    type Error = std::io::Error;
}

impl embedded_io_async::Write for SerialTx {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let mut serial = self.serial.lock().await;

        serial.write(buf)
    }
}

pub struct SerialRx {
    serial: Arc<Mutex<SerialStream>>,
}

impl embedded_io_async::ErrorType for SerialRx {
    type Error = std::io::Error;
}

impl embedded_io_async::Read for SerialRx {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut serial = self.serial.lock().await;

        serial.read(buf)
    }
}
