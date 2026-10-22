use super::*;
use base64::{Engine, engine::general_purpose::STANDARD};
use futures::executor::block_on;
use rsa::pkcs8::DecodePrivateKey;
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
    sync::OnceLock,
};

#[derive(Debug)]
struct Fake {
    input: VecDeque<u8>,
    output: Rc<RefCell<Vec<u8>>>,
    paused: Rc<Cell<bool>>,
    zero: bool,
}
impl Fake {
    fn new(messages: &[Message]) -> Self {
        let mut input = VecDeque::new();
        for message in messages {
            input.extend(message.header().as_bytes());
            input.extend(message.payload());
        }
        Self {
            input,
            output: Rc::default(),
            paused: Rc::default(),
            zero: true,
        }
    }
}
impl Transport for Fake {
    async fn read(&mut self, out: &mut [u8]) -> Result<usize> {
        if self.paused.get() {
            std::future::pending::<()>().await;
        }
        if self.zero {
            self.zero = false;
            return Ok(0);
        }
        let n = out.len().min(3).min(self.input.len());
        if n == 0 {
            return Err(invalid("Unexpected end of fake transport"));
        }
        for byte in &mut out[..n] {
            *byte = self.input.pop_front().unwrap();
        }
        Ok(n)
    }
    async fn write(&mut self, bytes: &[u8]) -> Result<()> {
        self.output.borrow_mut().extend_from_slice(bytes);
        Ok(())
    }
}
fn key() -> &'static WebUSBKey {
    static KEY: OnceLock<WebUSBKey> = OnceLock::new();
    KEY.get_or_init(|| WebUSBKey::generate().unwrap())
}
fn message(command: Command, a: u32, b: u32, data: &[u8]) -> Message {
    Message::try_new(command, a, b, data).unwrap()
}
fn connected() -> Message {
    message(Command::Cnxn, VERSION, 4096, b"device::\0")
}
fn written(bytes: &[u8]) -> Vec<Message> {
    let mut remaining = bytes;
    let mut messages = Vec::new();
    while !remaining.is_empty() {
        let header = Header::decode(&remaining[..24]).unwrap();
        let end = 24 + header.data_length() as usize;
        let packet = Message::from_header_and_payload(header, remaining[24..end].to_vec());
        assert!(packet.check_message_integrity());
        messages.push(packet);
        remaining = &remaining[end..];
    }
    messages
}

#[test]
fn auth_reuses_rsa_signing_and_sends_fixed_width_android_key() {
    block_on(async {
        let token = [7; 20];
        let fake = Fake::new(&[
            message(Command::Auth, AUTH_TOKEN, 0, &token),
            message(Command::Auth, AUTH_TOKEN, 0, &[9; 20]),
            connected(),
        ]);
        let output = fake.output.clone();
        Connection::connect(fake, key()).await.unwrap();
        let packets = written(&output.borrow());
        assert_eq!(packets.len(), 3);
        assert_eq!(packets[0].header().arg0(), VERSION);
        assert_eq!(packets[0].payload(), b"host::\0");
        assert_eq!(packets[1].header().arg0(), AUTH_SIGNATURE);
        let private = rsa::RsaPrivateKey::from_pkcs8_pem(&key().to_pkcs8_pem().unwrap()).unwrap();
        rsa::RsaPublicKey::from(&private)
            .verify(
                rsa::Pkcs1v15Sign::new::<sha1::Sha1>(),
                &token,
                packets[1].payload(),
            )
            .unwrap();
        assert_eq!(packets[2].header().arg0(), AUTH_RSAPUBLICKEY);
        assert_eq!(packets[2].payload().last(), Some(&0));
        let encoded = packets[2].payload().split(|&b| b == b' ').next().unwrap();
        let public = STANDARD.decode(encoded).unwrap();
        assert_eq!(public.len(), 524);
        assert_eq!(u32::from_le_bytes(public[..4].try_into().unwrap()), 64);
        let pem = key().to_pkcs8_pem().unwrap();
        assert_eq!(
            WebUSBKey::from_pkcs8_pem(&pem)
                .unwrap()
                .to_pkcs8_pem()
                .unwrap(),
            pem
        );
        assert!(!format!("{:?}", key()).contains("PRIVATE"));
    });
}

#[test]
fn fragmented_packets_close_and_sequential_streams() {
    block_on(async {
        let fake = Fake::new(&[
            connected(),
            message(Command::Okay, 7, 1, &[]),
            message(Command::Write, 7, 1, b"hello\n"),
            message(Command::Clse, 7, 1, &[]),
            message(Command::Okay, 8, 2, &[]),
            message(Command::Clse, 8, 2, &[]),
        ]);
        let output = fake.output.clone();
        let mut device = Connection::connect(fake, key()).await.unwrap();
        let mut first = device.open("shell:id").await.unwrap();
        assert_eq!(first.read().await.unwrap(), Some(b"hello\n".to_vec()));
        assert!(first.read().await.unwrap().is_none());
        drop(first);
        device.open("sync:").await.unwrap().close().await.unwrap();
        let packets = written(&output.borrow());
        let commands: Vec<_> = packets.iter().map(|p| p.header().command()).collect();
        assert_eq!(
            commands,
            [
                Command::Cnxn,
                Command::Open,
                Command::Okay,
                Command::Clse,
                Command::Open,
                Command::Clse
            ]
        );
        assert_eq!(packets[4].header().arg0(), 2);
    });
}

#[test]
fn writes_respect_peer_limit_and_hold_crossed_data_until_read() {
    block_on(async {
        let fake = Fake::new(&[
            message(Command::Cnxn, VERSION, 16, b"device::\0"),
            message(Command::Okay, 7, 1, &[]),
            message(Command::Write, 7, 1, b"crossed"),
            message(Command::Okay, 7, 1, &[]),
            message(Command::Okay, 7, 1, &[]),
            message(Command::Okay, 7, 1, &[]),
            message(Command::Clse, 7, 1, &[]),
        ]);
        let output = fake.output.clone();
        let mut device = Connection::connect(fake, key()).await.unwrap();
        let mut stream = device.open("sync:").await.unwrap();
        stream.write(&[1; 40]).await.unwrap();
        assert!(
            !written(&output.borrow())
                .iter()
                .any(|p| p.header().command() == Command::Okay)
        );
        assert_eq!(stream.read().await.unwrap(), Some(b"crossed".to_vec()));
        stream.close().await.unwrap();
        let packets = written(&output.borrow());
        let sizes: Vec<_> = packets
            .iter()
            .filter(|p| p.header().command() == Command::Write)
            .map(|p| p.payload().len())
            .collect();
        assert_eq!(sizes, [16, 16, 8]);
    });
}

#[test]
fn malformed_headers_are_rejected_before_payload_allocation() {
    block_on(async {
        for field in [0, 12, 20] {
            let mut fake = Fake::new(&[connected()]);
            let mut bytes: Vec<_> = fake.input.drain(..).collect();
            if field == 12 {
                bytes[12..16]
                    .copy_from_slice(&(u32::try_from(MAX_PAYLOAD).unwrap() + 1).to_le_bytes());
            } else {
                bytes[field] ^= 1;
            }
            fake.input = bytes.into();
            assert!(Connection::connect(fake, key()).await.is_err());
        }
        let mut fake = Fake::new(&[connected()]);
        *fake.input.back_mut().unwrap() ^= 1;
        assert!(Connection::connect(fake, key()).await.is_err());
    });
}

#[test]
fn rejected_auth_bad_stream_ids_and_dropped_streams_cannot_be_reused() {
    block_on(async {
        let fake = Fake::new(&[message(Command::Auth, AUTH_SIGNATURE, 0, &[0; 20])]);
        assert!(Connection::connect(fake, key()).await.is_err());
        let fake = Fake::new(&[
            connected(),
            message(Command::Okay, 7, 1, &[]),
            message(Command::Write, 7, 99, b"wrong"),
        ]);
        let mut device = Connection::connect(fake, key()).await.unwrap();
        let mut stream = device.open("shell:id").await.unwrap();
        assert!(stream.read().await.is_err());
        assert!(stream.read().await.is_err());
        drop(stream);
        assert!(device.open("shell:id").await.is_err());
    });
}

#[test]
fn cancelled_read_or_open_requires_reconnect_and_bad_names_do_not_send() {
    use std::future::Future;
    use std::{
        pin::pin,
        task::{Context, Poll, Waker},
    };
    block_on(async {
        for cancel_open in [false, true] {
            let fake = Fake::new(&[connected(), message(Command::Okay, 7, 1, &[])]);
            let paused = fake.paused.clone();
            let mut device = Connection::connect(fake, key()).await.unwrap();
            for bad in ["", "shell:x\0y"] {
                assert!(device.open(bad).await.is_err());
            }
            if cancel_open {
                paused.set(true);
                {
                    let mut open = pin!(device.open("shell:id"));
                    assert!(matches!(
                        open.as_mut().poll(&mut Context::from_waker(Waker::noop())),
                        Poll::Pending
                    ));
                }
            } else {
                let mut stream = device.open("shell:id").await.unwrap();
                paused.set(true);
                {
                    let mut read = pin!(stream.read());
                    assert!(
                        read.as_mut()
                            .poll(&mut Context::from_waker(Waker::noop()))
                            .is_pending()
                    );
                }
                assert!(stream.read().await.is_err());
                drop(stream);
            }
            assert!(device.open("shell:id").await.is_err());
        }
    });
}
