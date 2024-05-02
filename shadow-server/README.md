# Shadow Server

## Directories

### Network

*Containing the communicate code between server and client*

Shadow uses dual-way RPC to connect server and client, which is defined in `server.rs`.

Functions that handle TCP connection is defined in `run.rs`

### Web

*Containing the web interface*

Shadow uses `warp` to handle HTTP request issued by user and dispatch it using `server_objs` which is a collection of running client instances

### Misc

*Version and other stuffs*
