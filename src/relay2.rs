use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};

use bincode::{Decode, Encode};

use tokio::{net::UdpSocket, time::sleep};

#[derive(Clone)]
pub struct RemoteWriter {
    socket: Arc<UdpSocket>,
    
    latest_send: usize,
}
pub struct RemoteReader{
    socket: Arc<UdpSocket>,
    buf: HashMap<usize, HashMap<usize, Vec<u8>>>,
    latest_read: usize,
}

const CHUNK_SIZE: usize = 60000;
pub async fn get_remote() -> (RemoteWriter, RemoteReader) {

    let sock = UdpSocket::bind("0.0.0.0:9876").await.ok().unwrap();
    #[cfg(target_os = "android")]
    let remote_addr = "172.28.93.183:9876";
    #[cfg(not(target_os = "android"))]
    let remote_addr = "172.28.108.180:9876";

    let _ = sock.connect(remote_addr).await;

    let socket = Arc::new(sock);
    log::warn!("connected");
    (RemoteWriter {
        socket: socket.clone(),
        latest_send: 0,
    }, RemoteReader {
        socket,
        buf: HashMap::new(),
        latest_read: 0,
    })

    
}

pub async fn send_message(r: &mut RemoteWriter, msg: Message) {
    log::error!("Sending message: {:?}", msg);
    let data = bincode::encode_to_vec(msg, bincode::config::standard()).unwrap();
    let to_send: Vec<(usize, &[u8])> = data.as_slice().chunks(CHUNK_SIZE).enumerate().collect();
    let to_send_len = to_send.len();
    r.latest_send+=1;
    
    let to_send_id = r.latest_send;
    for (i, chunk) in to_send.iter() {
        let packet = Chunk {
            packet_id: to_send_id,
            packet_count: to_send_len,
            chunk_index: *i,
            data: chunk.to_vec(),
        };
        let data = bincode::encode_to_vec(packet, bincode::config::standard()).unwrap();
        let e = r.socket.send(data.as_slice()).await;
        if e.is_err() {
            log::error!("Error sending data to remote: {:?}", e);
        }
    }
}

pub async fn recv_messages(r: &mut RemoteReader) -> Option<Message> {
    let mut ret = None;
    //let data = bincode::encode_to_vec(msg, bincode::config::standard()).unwrap();
    let mut buf = [0; CHUNK_SIZE * 11 / 10];
    while let Ok(x) = r.socket.try_recv(&mut buf) {
        if let Ok((chunk, _)) =
            bincode::decode_from_slice::<Chunk, _>(&buf[0..x], bincode::config::standard())
        {   
            log::error!("Received chunk:");
            let t = r.buf.entry(chunk.packet_id).or_insert(Default::default());
            t.insert(chunk.chunk_index, chunk.data);
            if t.len() == chunk.packet_count {
                log::error!("all parts: {:?}", t.len());
                let mut data = Vec::new();
                for i in 0..chunk.packet_count {
                    if let Some(v) = t.remove(&i) {
                        data.extend(v);
                    }
                }
                let msg =
                    bincode::decode_from_slice::<Message, _>(&data, bincode::config::standard())
                        .ok()
                        .map(|x| x.0);
                println!("Decoded message: {:?} {} {}", msg, r.latest_read, chunk.packet_id);
                
                if msg.is_some() {
                    ret = msg;
                    r.latest_read = chunk.packet_id;
                }
                /*if msg.is_some() && r.latest_read < chunk.packet_id {
                    log::error!("Received message:");
                    r.latest_read = chunk.packet_id;
                    ret = msg;
                }*/
                r.buf.remove(&chunk.packet_id);
            }
        }
        
    }
    sleep(Duration::from_millis(30)).await;
    if r.buf.len() > 10 {
        let from = r.latest_read;
        r.buf.retain(|k, _| *k > from);
    }

    ret
}

#[derive(Encode, Decode)]
struct Chunk {
    packet_id: usize,
    packet_count: usize,
    chunk_index: usize,
    data: Vec<u8>,
}

#[derive(Encode, Decode)]
pub enum Message {
    Motors(f32, f32),
    Connect {
        addr: String,
        service: u128,
        char: u128,
    },
    Img(Vec<u8>),
}
impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Motors(x, y) => write!(f, "Motors: {}, {}", x, y),
            Message::Connect { .. } => write!(f, "Connect"),
            Message::Img(x) => write!(f, "Img: {}", x.len()),
        }
    }
}
