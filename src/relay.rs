use std::{collections::{HashMap, HashSet}, pin::pin, sync::{mpsc::{self, Receiver, Sender}, Arc}, time::Duration};

use bincode::{Decode, Encode};
use btleplug::api::BDAddr;
use futures::{pin_mut, select, FutureExt, SinkExt};
use log::{error, info};
use matchbox_socket::{ChannelConfig, PeerId, PeerState, RtcIceServerConfig, WebRtcSocket, WebRtcSocketBuilder};
use tokio::{spawn, sync::Mutex, time::sleep};
static CHANNEL_ID: usize = 0;
pub struct Remote {
    sender: Sender<Box<[u8]>>, 
    reciver: Receiver<Box<[u8]>>,
    buf: HashMap<usize, HashMap<usize, Vec<u8>>>,
    how_many_queued: Arc<Mutex<usize>>,
    latest_read: Arc<Mutex<usize>>,
    latest_send: Arc<Mutex<usize>>,
}
pub async fn get_remote()->Remote{
    let (send_to_remote, recv_send_to_remote) = mpsc::channel::<Box<[u8]>>();
    let (send_recv_from_remote, recv_from_remote) = mpsc::channel::<Box<[u8]>>();
    let latest_read = Arc::new(Mutex::new(0));
    let latest_send = Arc::new(Mutex::new(0));
    let how_many_queued = Arc::new(Mutex::new(0));

    let latest_read_int = latest_read.clone();
    let latest_send_int = latest_send.clone();
    let how_many_queued_int = how_many_queued.clone();

    spawn(async move{
        info!("Connecting to matchbox");
        let config = ChannelConfig{
            ordered: false,
            max_retransmits: None,
        };
        let (mut socket, loop_fut) = WebRtcSocketBuilder::new("ws://insigno.mindshub.it:3536/")
            .add_channel(config)
            .build();
        //let (mut socket, loop_fut) = WebRtcSocket::new_unreliable("ws://insigno.mindshub.it:3536/");

        let loop_fut = loop_fut.fuse();
        //spawn(loop_fut);
        futures::pin_mut!(loop_fut);

        let timeout = Duration::from_millis(100);
        futures::pin_mut!(timeout);
        let mut peers: HashSet<PeerId>;
        loop {
            peers = socket.connected_peers().collect();
            if peers.is_empty() {
                *latest_read_int.lock().await = 0;
                *latest_send_int.lock().await = 0;
            }
            // Handle any new peers
            for (peer, state) in socket.update_peers() {
                match state {
                    PeerState::Connected => {
                        error!("Peer joined: {peer}");
                        peers.insert(peer);
                        /*let packet = "hello friend!".as_bytes().to_vec().into_boxed_slice();
                        socket.channel_mut(CHANNEL_ID).send(packet, peer);*/
                    }
                    PeerState::Disconnected => {
                        peers.remove(&peer);
                        error!("Peer left: {peer}");
                    }
                }
            }
            
            while let Some(packet) = recv_send_to_remote.try_recv().ok() {
                *how_many_queued_int.lock().await-=1;
                // Send any messages to the remote peer
                for peer in peers.iter() {
                    socket.channel_mut(CHANNEL_ID).send(packet.clone(), *peer);
                }
            }
            // Accept any messages incoming
            for (peer, packet) in socket.channel_mut(CHANNEL_ID).receive() {
                //log::error!("Received packet from peer {peer}: {}", packet.len());
                let _ = send_recv_from_remote.send(packet).unwrap();
            }
            let c = socket.channel_mut(CHANNEL_ID).flush().await;
            //println!("Flushed channel: {:?}", c);
            
            select! {
                // Restart this loop every 100ms
                _ = sleep(*timeout).fuse() => {
                    //timeout.reset(Duration::from_millis(100));
                }

                // Or break if the message loop ends (disconnected, closed, etc.)
                _ = &mut loop_fut => {
                    break;
                }
            }
        }
    });
    sleep(Duration::from_secs(1)).await;
    Remote{
        sender: send_to_remote,
        reciver: recv_from_remote,
        buf: HashMap::new(),
        latest_read: latest_read,
        latest_send: latest_send,
        how_many_queued: how_many_queued,
    }
}

pub async fn send_message(r: &mut Remote, msg: Message) {
    if *r.how_many_queued.lock().await > 1{
        return;
    }
    let data = bincode::encode_to_vec(msg, bincode::config::standard()).unwrap();
    let to_send: Vec<(usize, &[u8])>= data.as_slice().chunks(60000).enumerate().collect();
    let to_send_len = to_send.len();
    let mut latest_send = r.latest_send.lock().await;
    *latest_send += 1;
    let to_send_id = *latest_send;
    for (i, chunk) in to_send.iter(){
        let packet = Chunk{
            packet_id: to_send_id,
            packet_count: to_send_len,
            chunk_index: *i,
            data: chunk.to_vec(),
        };
        let data = bincode::encode_to_vec(packet, bincode::config::standard()).unwrap();
        *r.how_many_queued.lock().await += 1;
        let _ = r.sender.send(data.into());
    }
}

pub async fn recv_messages(r: &mut Remote) -> Option<Message> {

    let mut ret=None;
    //let data = bincode::encode_to_vec(msg, bincode::config::standard()).unwrap();
    while let Ok(x) = r.reciver.try_recv(){
        if let Ok((chunk, _)) = bincode::decode_from_slice::<Chunk, _>(&x, bincode::config::standard()){
            let t = r.buf.entry(chunk.packet_id).or_insert(Default::default());
            t.insert(chunk.chunk_index, chunk.data);
            if t.len() == chunk.packet_count {
                let mut data = Vec::new();
                for i in 0..chunk.packet_count {
                    if let Some(v) = t.remove(&i){
                        data.extend(v);
                    }
                }
                let msg =  bincode::decode_from_slice::<Message, _>(&data, bincode::config::standard()).ok().map(|x| x.0);
                let mut latest_read = r.latest_read.lock().await;
                if msg.is_some() && *latest_read < chunk.packet_id{
                    *latest_read = chunk.packet_id;
                    ret = msg;
                }
                r.buf.contains_key(&chunk.packet_id);
            }
        }
    }
    if r.buf.len() > 10{
        let from = *r.latest_read.lock().await;
        r.buf.retain(|k, _| *k > from);
    }

    ret
}

#[derive(Encode, Decode)]
struct Chunk{
    packet_id: usize,
    packet_count: usize,
    chunk_index: usize,
    data: Vec<u8>,
}

#[derive(Encode, Decode)]
pub enum Message {
    Motors(f32, f32),
    Connect{
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
            Message::Connect{..} => write!(f, "Connect"),
            Message::Img(x) => write!(f, "Img: {}", x.len()),
        }
    }
}
