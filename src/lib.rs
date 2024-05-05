#![cfg_attr(not(test), no_std)]

mod command;
mod setting;

use core::future::poll_fn;
use core::task::Poll;

use command::{command, Event, Response, MAX_PAYLOAD_LEN, START};
use embedded_io_async::{Read, Write};
use heapless::spsc::{Consumer, Producer, Queue};
use heapless::Vec;

#[derive(Debug)]
pub struct ResponseFrame {
    command: Response,
    data: Vec<u8, MAX_PAYLOAD_LEN>,
}

#[derive(Debug)]
pub struct EventFrame {
    command: Event,
    data: Vec<u8, MAX_PAYLOAD_LEN>,
}

pub struct Radio<'a, W>
where
    W: Write,
{
    serial: W,
    response: Consumer<'a, ResponseFrame, 2>,
    event: Consumer<'a, EventFrame, 16>,
}

impl<'a, W> Radio<'a, W>
where
    W: Write,
{
    pub fn new<R: Read>(
        writer: W,
        reader: R,
        response_queue: &'a mut Queue<ResponseFrame, 2>,
        event_queue: &'a mut Queue<EventFrame, 16>,
    ) -> (Self, Ingress<'a, R>) {
        let (response_producer, response_consumer) = response_queue.split();
        let (event_producer, event_consumer) = event_queue.split();

        (
            Self {
                serial: writer,
                response: response_consumer,
                event: event_consumer,
            },
            Ingress {
                serial: reader,
                response: response_producer,
                event: event_producer,
            },
        )
    }

    pub async fn reset(&mut self) -> Result<(), W::Error> {
        let mut buf = [0; 224];
        let size = command(&mut buf, command::Request::Reset, &[]);
        self.serial.write(&buf[..size]).await?;

        poll_fn(|cx| {
            if let Some(_response) = self.response.dequeue() {
                Poll::Ready(Ok(()))
            } else {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        })
        .await
    }
}

pub struct Ingress<'a, S>
where
    S: Read,
{
    serial: S,
    response: Producer<'a, ResponseFrame, 2>,
    event: Producer<'a, EventFrame, 16>,
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
                    .enqueue(EventFrame {
                        command: event,
                        data: payload,
                    })
                    .ok();
                continue;
            }

            if let Some(response) = Response::try_from_raw(cmd) {
                self.response
                    .enqueue(ResponseFrame {
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
