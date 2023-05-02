use std::convert::TryInto;

use async_std::os::unix::net::UnixStream;
use async_std::prelude::*;
use async_std::task::spawn;
use async_tungstenite::tungstenite::{protocol::Role, Message};
use async_tungstenite::WebSocketStream;
use base64::Engine;
use futures_lite::future::race;
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info};
use nbd_netlink::{NBDConnect, NBD};
use serde::Deserialize;
use sha1::{digest::Update, Digest, Sha1};
use tide::http::format_err;
use tide::http::headers::{HeaderName, CONNECTION, UPGRADE};
use tide::http::upgrade::Connection;
use tide::{Request, Response, StatusCode};

const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

const NBD_CMD_READ: u16 = 0;
const NBD_CMD_DISC: u16 = 2;

#[derive(Deserialize)]
struct Query {
    size: u64,
}

async fn handle_connection(ws: WebSocketStream<Connection>, nbd_size: u64) {
    let nbd = {
        let (nbd_kernel, nbd_web) = UnixStream::pair().unwrap();

        let mut nbd = NBD::new().unwrap();
        let index = NBDConnect::new()
            .disconnect_on_close(true)
            .size_bytes(nbd_size)
            .read_only(true)
            .connect(&mut nbd, &[nbd_kernel])
            .unwrap();

        info!("nbd: set up device: nbd{index} ({nbd_size} bytes)");

        nbd_web
    };

    let (mut nbd_rx, mut nbd_tx) = (&nbd, &nbd);
    let (mut ws_tx, mut ws_rx) = ws.split();

    let nbd_to_ws_task = async {
        let mut header = [0u8; 28];

        while let Ok(_) = nbd_rx.read_exact(&mut header).await {
            let req_type = u16::from_be_bytes(header[6..8].try_into().unwrap());

            if req_type == NBD_CMD_DISC {
                info!("nbd: disconnect request from kernel");
                ws_tx.close().await.unwrap();
                break;
            }

            if req_type != NBD_CMD_READ {
                error!("nbd: unsupported type {req_type}");
                break;
            }

            if ws_tx.send(Message::binary(header)).await.is_err() {
                break;
            }
        }
    };

    let ws_to_nbd_task = async {
        while let Some(Ok(Message::Binary(data))) = ws_rx.next().await {
            nbd_tx.write_all(&data).await.unwrap();
        }
    };

    race(nbd_to_ws_task, ws_to_nbd_task).await;

    info!("nbd: done");
}

fn header_contains_ignore_case(req: &Request<()>, header_name: HeaderName, value: &str) -> bool {
    req.header(header_name)
        .map(|h| {
            h.as_str()
                .split(',')
                .any(|s| s.trim().eq_ignore_ascii_case(value.trim()))
        })
        .unwrap_or(false)
}

pub(super) fn register(server: &mut tide::Server<()>) {
    server.at("/v1/nbd").get(move |req: Request<()>| {
        async move {
            // These are the good parts from tide-websockets without the bad
            // WebSocketConnection wrapper.

            let connection_upgrade = header_contains_ignore_case(&req, CONNECTION, "upgrade");
            let upgrade_to_websocket = header_contains_ignore_case(&req, UPGRADE, "websocket");
            let upgrade_requested = connection_upgrade && upgrade_to_websocket;

            if !upgrade_requested {
                return Ok(Response::new(StatusCode::UpgradeRequired));
            }

            let header = match req.header("Sec-Websocket-Key") {
                Some(h) => h.as_str(),
                None => return Err(format_err!("expected sec-websocket-key")),
            };

            let protocol = req.header("Sec-Websocket-Protocol").and_then(|value| {
                value
                    .as_str()
                    .split(',')
                    .map(str::trim)
                    .find(|req_p| req_p == &"nbd")
            });

            let nbd_size = req.query::<Query>().unwrap().size;

            let mut response = Response::new(StatusCode::SwitchingProtocols);

            response.insert_header(UPGRADE, "websocket");
            response.insert_header(CONNECTION, "Upgrade");
            let hash = Sha1::new().chain(header).chain(WEBSOCKET_GUID).finalize();
            let hash = base64::engine::general_purpose::STANDARD.encode(&hash[..]);
            response.insert_header("Sec-Websocket-Accept", hash);
            response.insert_header("Sec-Websocket-Version", "13");

            if let Some(protocol) = protocol {
                response.insert_header("Sec-Websocket-Protocol", protocol);
            }

            let http_res: &mut tide::http::Response = response.as_mut();
            let upgrade_receiver = http_res.recv_upgrade().await;

            spawn(async move {
                if let Some(stream) = upgrade_receiver.await {
                    let ws = WebSocketStream::from_raw_socket(stream, Role::Server, None).await;
                    handle_connection(ws, nbd_size).await;
                }
            });

            Ok(response)
        }
    });
}
