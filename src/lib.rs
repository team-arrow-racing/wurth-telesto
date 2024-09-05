#![cfg_attr(not(any(test, feature = "cli")), no_std)]

mod command;
mod setting;

use core::future::poll_fn;
use core::task::Poll;

pub use command::{Event, Mode, Response};

use command::{command, Request, SendDataError, MAX_PAYLOAD_LEN, START};
use embedded_io_async::{Read, Write};
use heapless::spsc::{Consumer, Producer, Queue};
use heapless::Vec;

/// Command/response frame.
#[derive(Debug)]
pub struct Frame<T> {
    command: T,
    data: Vec<u8, MAX_PAYLOAD_LEN>,
}

impl<T> Frame<T> {
    pub fn command(&self) -> &T {
        &self.command
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// Error kind.
#[derive(Debug)]
pub enum Error<S, IO> {
    Status(S),
    Io(IO),
}

/// Radio module instance.
pub struct Radio<'a, W>
where
    W: Write,
{
    serial: W,
    response: Consumer<'a, Frame<Response>, 2>,
    event: Consumer<'a, Frame<Event>, 16>,
}

impl<'a, W> Radio<'a, W>
where
    W: Write,
{
    pub fn new<R: Read>(
        writer: W,
        reader: R,
        response_queue: &'a mut Queue<Frame<Response>, 2>,
        event_queue: &'a mut Queue<Frame<Event>, 16>,
    ) -> (Self, Ingress<'a, R>) {
        let (response_producer, response_consumer) = response_queue.split();
        let (event_producer, event_consumer) = event_queue.split();

        (
            Self {
                serial: writer,
                response: response_consumer,
                event: event_consumer,
            },
            Ingress::<'a> {
                serial: reader,
                response: response_producer,
                event: event_producer,
            },
        )
    }

    /// Poll until an event is received.
    pub async fn poll_event(&mut self) -> Frame<Event> {
        poll_fn(|cx| {
            if let Some(event) = self.event.dequeue() {
                Poll::Ready(event)
            } else {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        })
        .await
    }

    /// Send data command.
    ///
    /// Panics if the data length is larger than 220 (the maximum allowed payload).
    pub async fn send(&mut self, data: &[u8]) -> Result<(), Error<SendDataError, W::Error>> {
        assert!(data.len() <= 220);

        let mut buf = [0; 224];
        let size = command(&mut buf, Request::SendData, data);
        self.serial.write(&buf[..size]).await.map_err(Error::Io)?;

        let response = self.poll_response().await;
        let status = response.data[0];

        if status == 0 {
            Ok(())
        } else {
            Err(Error::Status(status.into()))
        }
    }

    /// Performs a soft-reset of the radio module.
    ///
    /// Returns [`Ok`] once the reset has been confirmed by the device.
    pub async fn reset(&mut self) -> Result<(), Error<(), W::Error>> {
        let mut buf = [0; 224];
        let size = command(&mut buf, command::Request::Reset, &[]);
        self.serial.write(&buf[..size]).await.map_err(Error::Io)?;

        let response = self.poll_response().await;
        let status = response.data[0];

        if status == 0 {
            Ok(())
        } else {
            Err(Error::Status(()))
        }
    }

    /// Performs a factory reset of the radio module.
    pub async fn factory_reset(&mut self) -> Result<(), Error<(), W::Error>> {
        let mut buf = [0; 224];
        let size = command(&mut buf, command::Request::FactoryReset, &[]);
        self.serial.write(&buf[..size]).await.map_err(Error::Io)?;

        let response = self.poll_response().await;
        let status = response.data[0];

        if status == 0 {
            Ok(())
        } else {
            Err(Error::Status(()))
        }
    }

    /// Enters the radio into standby mode.
    ///
    /// Returns [`Ok`] confirming the device will enter standby.
    pub async fn standby(&mut self) -> Result<(), Error<(), W::Error>> {
        let mut buf = [0; 224];
        let size = command(&mut buf, command::Request::Standby, &[]);
        self.serial.write(&buf[..size]).await.map_err(Error::Io)?;

        let response = self.poll_response().await;
        let status = response.data[0];

        if status == 0 {
            Ok(())
        } else {
            Err(Error::Status(()))
        }
    }

    /// Gets the receive signal strength (RSSI) of the last packet received.
    pub async fn rssi(&mut self) -> Result<u8, Error<(), W::Error>> {
        let mut buf = [0; 224];
        let size = command(&mut buf, command::Request::Rssi, &[]);
        self.serial.write(&buf[..size]).await.map_err(Error::Io)?;

        let response = self.poll_response().await;
        let status = response.data[0];

        Ok(status)
    }

    /// Set the transmit power.
    ///
    /// A value outisde the allowable range will result in an error response.
    pub async fn tx_power(&mut self, power: u8) -> Result<(), Error<(), W::Error>> {
        let mut buf = [0; 224];
        let size = command(&mut buf, command::Request::TransmitPower, &[power]);
        self.serial.write(&buf[..size]).await.map_err(Error::Io)?;

        let response = self.poll_response().await;
        let status = response.data[0];

        if status == power {
            Ok(())
        } else {
            Err(Error::Status(()))
        }
    }

    /// Set the channel.
    pub async fn channel(&mut self, channel: u8) -> Result<(), Error<(), W::Error>> {
        let mut buf = [0; 224];
        let size = command(&mut buf, command::Request::SetChannel, &[channel]);
        self.serial.write(&buf[..size]).await.map_err(Error::Io)?;

        let response = self.poll_response().await;
        let status = response.data[0];

        if status == channel {
            Ok(())
        } else {
            Err(Error::Status(()))
        }
    }

    /// Set destination net ID.
    pub async fn destination_net(&mut self, id: u8) -> Result<(), Error<(), W::Error>> {
        let mut buf = [0; 224];
        let size = command(&mut buf, command::Request::SetDestinationNetworkId, &[id]);
        self.serial.write(&buf[..size]).await.map_err(Error::Io)?;

        let response = self.poll_response().await;
        let status = response.data[0];

        if status == 0x00 {
            Ok(())
        } else {
            Err(Error::Status(()))
        }
    }

    /// Set destination address.
    pub async fn destination_address(&mut self, address: u8) -> Result<(), Error<(), W::Error>> {
        let mut buf = [0; 224];
        let size = command(
            &mut buf,
            command::Request::SetDestinationAddress,
            &[address],
        );
        self.serial.write(&buf[..size]).await.map_err(Error::Io)?;

        let response = self.poll_response().await;
        let status = response.data[0];

        if status == 0x00 {
            Ok(())
        } else {
            Err(Error::Status(()))
        }
    }

    /// Set operating mode.
    ///
    /// The mode change is performed after the achnoledge response is transmitted.
    pub async fn mode(&mut self, mode: Mode) -> Result<(), Error<(), W::Error>> {
        let mut buf = [0; 224];
        let size = command(&mut buf, command::Request::SetMode, &[mode as u8]);
        self.serial.write(&buf[..size]).await.map_err(Error::Io)?;

        let response = self.poll_response().await;
        let status = response.data[0];

        if status == 0x00 {
            Ok(())
        } else {
            Err(Error::Status(()))
        }
    }

    /// Poll until a response frame is received through the response channel.
    async fn poll_response(&mut self) -> Frame<Response> {
        poll_fn(|cx| {
            if let Some(response) = self.response.dequeue() {
                Poll::Ready(response)
            } else {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        })
        .await
    }
}

/// Incomming data handler.
pub struct Ingress<'a, S>
where
    S: Read,
{
    serial: S,
    response: Producer<'a, Frame<Response>, 2>,
    event: Producer<'a, Frame<Event>, 16>,
}

impl<'a, S> Ingress<'a, S>
where
    S: Read,
{
    pub async fn ingest(&mut self) -> Result<(), IngestError> {
        loop {
            let mut buf = [0; 3];
            self.serial.read_exact(&mut buf).await.ok();

            if buf[0] != START {
                continue;
            }

            let cmd = buf[1];
            let len = buf[2] as usize;

            if len > MAX_PAYLOAD_LEN {
                return Err(IngestError::PayloadLength);
            }

            let mut payload = Vec::<u8, MAX_PAYLOAD_LEN>::new();
            unsafe { payload.set_len(len) };
            self.serial.read_exact(&mut payload[0..len]).await.ok();

            let mut _checksum = [0; 1];
            self.serial.read_exact(&mut buf).await.ok();

            //todo: check checksum

            if let Some(event) = Event::try_from_raw(cmd) {
                self.event
                    .enqueue(Frame::<Event> {
                        command: event,
                        data: payload,
                    })
                    .ok();
                continue;
            }

            if let Some(response) = Response::try_from_raw(cmd) {
                self.response
                    .enqueue(Frame::<Response> {
                        command: response,
                        data: payload,
                    })
                    .ok();
                continue;
            }
        }
    }
}

/// Ingest error.
#[derive(Debug)]
pub enum IngestError {
    /// Start byte was not correct.
    StartByte,
    /// Payload length is too long.
    PayloadLength,
    /// Command id is not recognised.
    UnknownCommand,
}
