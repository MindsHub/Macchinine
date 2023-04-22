# Setup

## Per installare Rust

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Per eseguire Rust

```
sudo apt install libdbus-1-dev pkg-config libudev-dev
cargo run
```

## MAC

- `A8:10:87:67:73:2A` quella modificata con la batteria grossa, con l'HC-08
- `48:87:2D:11:A6:F1` quella con le batteria blu (non modificata), il bluetooth strano

## Per connettersi

```
bluetoothctl
$ scan on # poi aspettare che compaia il mac della macchinina
$ pair <MAC> # non serve se si ha gia' fatto il pairing
$ connect <MAC> # non serve se si ha eseguito il comando sopra
```
