use std::{net::SocketAddr, time::Duration};

use bytes::Bytes;
use futures_util::{self, SinkExt, StreamExt, TryStreamExt};
use http::{StatusCode, Version};
use serde_json::Value;
use tokio::sync::{
  mpsc::{self, UnboundedReceiver, UnboundedSender},
  oneshot,
};
use wreq::{
  header::{HeaderMap, HeaderValue},
  ws::{
    self,
    message::{self, CloseCode, CloseFrame, Utf8Bytes},
    WebSocketResponse,
  },
};

use crate::error::Error;

/// A WebSocket message wrapper.
#[derive(Clone, Debug)]
pub struct Message(pub message::Message);

impl Message {
  pub fn json(&self) -> Result<Value, Error> {
    self.0.json::<Value>().map_err(Error::Library)
  }

  pub fn data(&self) -> Option<Bytes> {
    match &self.0 {
      message::Message::Text(text) => Some(text.clone().into()),
      message::Message::Binary(bytes)
      | message::Message::Ping(bytes)
      | message::Message::Pong(bytes) => Some(bytes.clone()),
      _ => None,
    }
  }

  pub fn text(&self) -> Option<&str> {
    match &self.0 {
      message::Message::Text(text) => Some(text.as_ref()),
      _ => None,
    }
  }

  pub fn binary(&self) -> Option<&Bytes> {
    match &self.0 {
      message::Message::Binary(bytes) => Some(bytes),
      _ => None,
    }
  }

  pub fn ping(&self) -> Option<&Bytes> {
    match &self.0 {
      message::Message::Ping(bytes) => Some(bytes),
      _ => None,
    }
  }

  pub fn pong(&self) -> Option<&Bytes> {
    match &self.0 {
      message::Message::Pong(bytes) => Some(bytes),
      _ => None,
    }
  }

  pub fn close(&self) -> Option<(u16, Option<&str>)> {
    match &self.0 {
      message::Message::Close(Some(frame)) => {
        Some((u16::from(frame.code.clone()), Some(frame.reason.as_ref())))
      }
      _ => None,
    }
  }

  pub fn from_text(text: String) -> Self {
    Self(message::Message::text(text))
  }

  pub fn from_binary(data: Bytes) -> Self {
    Self(message::Message::binary(data))
  }

  pub fn from_ping(data: Bytes) -> Self {
    Self(message::Message::ping(data))
  }

  pub fn from_pong(data: Bytes) -> Self {
    Self(message::Message::pong(data))
  }

  pub fn from_close(code: u16, reason: Option<String>) -> Self {
    let reason = reason
      .map(|s| Bytes::from(s.into_bytes()))
      .and_then(|bytes| Utf8Bytes::try_from(bytes).ok())
      .unwrap_or_else(|| Utf8Bytes::from_static("Goodbye"));
    Self(message::Message::close(CloseFrame {
      code: CloseCode::from(code),
      reason,
    }))
  }

  pub fn from_json_text(json: &Value) -> Result<Self, Error> {
    message::Message::text_from_json(json)
      .map(Message)
      .map_err(Error::Library)
  }

  pub fn from_json_binary(json: &Value) -> Result<Self, Error> {
    message::Message::binary_from_json(json)
      .map(Message)
      .map_err(Error::Library)
  }

  pub fn into_inner(self) -> message::Message {
    self.0
  }
}

/// Binding-agnostic WebSocket wrapper.
#[derive(Clone)]
pub struct WebSocket {
  version: Version,
  status: StatusCode,
  remote_addr: Option<SocketAddr>,
  local_addr: Option<SocketAddr>,
  headers: HeaderMap,
  protocol: Option<HeaderValue>,
  cmd: UnboundedSender<Command>,
}

impl WebSocket {
  pub async fn new(response: WebSocketResponse) -> Result<Self, Error> {
    let version = response.version();
    let status = response.status();
    let remote_addr = response.remote_addr();
    let local_addr = response.local_addr();
    let headers = response.headers().clone();
    let websocket = response.into_websocket().await.map_err(Error::Library)?;
    let protocol = websocket.protocol().cloned();
    let (cmd, rx) = mpsc::unbounded_channel();
    tokio::spawn(command_task(websocket, rx));

    Ok(Self {
      version,
      status,
      remote_addr,
      local_addr,
      headers,
      protocol,
      cmd,
    })
  }

  pub fn version(&self) -> Version {
    self.version
  }

  pub fn status(&self) -> StatusCode {
    self.status
  }

  pub fn remote_addr(&self) -> Option<SocketAddr> {
    self.remote_addr
  }

  pub fn local_addr(&self) -> Option<SocketAddr> {
    self.local_addr
  }

  pub fn headers(&self) -> &HeaderMap {
    &self.headers
  }

  pub fn protocol(&self) -> Option<&HeaderValue> {
    self.protocol.as_ref()
  }

  pub async fn recv(&self, timeout: Option<Duration>) -> Result<Option<Message>, Error> {
    recv(self.cmd.clone(), timeout).await
  }

  pub async fn send(&self, message: Message) -> Result<(), Error> {
    send(self.cmd.clone(), message).await
  }

  pub async fn send_all(&self, messages: Vec<Message>) -> Result<(), Error> {
    send_all(self.cmd.clone(), messages).await
  }

  pub async fn close(&self, code: Option<u16>, reason: Option<String>) -> Result<(), Error> {
    close(self.cmd.clone(), code, reason).await
  }
}

enum Command {
  Send(Message, oneshot::Sender<Result<(), Error>>),
  SendMany(Vec<Message>, oneshot::Sender<Result<(), Error>>),
  Recv(
    Option<Duration>,
    oneshot::Sender<Result<Option<Message>, Error>>,
  ),
  Close(
    Option<u16>,
    Option<String>,
    oneshot::Sender<Result<(), Error>>,
  ),
}

async fn send_command<T>(
  cmd: UnboundedSender<Command>,
  make: impl FnOnce(oneshot::Sender<Result<T, Error>>) -> Command,
) -> Result<T, Error> {
  if cmd.is_closed() {
    return Err(Error::WebSocketDisconnected);
  }
  let (tx, rx) = oneshot::channel();
  cmd
    .send(make(tx))
    .map_err(|_| Error::WebSocketDisconnected)?;
  match rx.await {
    Ok(res) => res,
    Err(_) => Err(Error::WebSocketDisconnected),
  }
}

async fn recv(
  cmd: UnboundedSender<Command>,
  timeout: Option<Duration>,
) -> Result<Option<Message>, Error> {
  send_command(cmd, |tx| Command::Recv(timeout, tx)).await
}

async fn send(cmd: UnboundedSender<Command>, message: Message) -> Result<(), Error> {
  send_command(cmd, |tx| Command::Send(message, tx)).await
}

async fn send_all(cmd: UnboundedSender<Command>, messages: Vec<Message>) -> Result<(), Error> {
  if messages.is_empty() {
    return Ok(());
  }
  send_command(cmd, |tx| Command::SendMany(messages, tx)).await
}

async fn close(
  cmd: UnboundedSender<Command>,
  code: Option<u16>,
  reason: Option<String>,
) -> Result<(), Error> {
  send_command(cmd, |tx| Command::Close(code, reason, tx)).await
}

async fn command_task(ws: ws::WebSocket, mut rx: UnboundedReceiver<Command>) {
  let (mut writer, mut reader) = ws.split();
  while let Some(command) = rx.recv().await {
    match command {
      Command::Send(message, tx) => {
        let res = writer.send(message.0).await.map_err(Error::Library);
        let _ = tx.send(res);
      }
      Command::SendMany(messages, tx) => {
        let mut stream = futures_util::stream::iter(messages.into_iter().map(|m| Ok(m.0)));
        let res = writer.send_all(&mut stream).await.map_err(Error::Library);
        let _ = tx.send(res);
      }
      Command::Recv(timeout, tx) => {
        let fut = async {
          reader
            .try_next()
            .await
            .map(|opt| opt.map(Message))
            .map_err(Error::Library)
        };

        let res = if let Some(timeout) = timeout {
          match tokio::time::timeout(timeout, fut).await {
            Ok(res) => res,
            Err(err) => Err(Error::Timeout(err)),
          }
        } else {
          fut.await
        };
        let _ = tx.send(res);
      }
      Command::Close(code, reason, tx) => {
        let code = code.map(CloseCode::from).unwrap_or(CloseCode::NORMAL);
        let reason = reason
          .map(|s| Bytes::from(s.into_bytes()))
          .and_then(|bytes| Utf8Bytes::try_from(bytes).ok());
        let frame = reason.map(|reason| CloseFrame { code, reason });

        let res = writer
          .send(message::Message::Close(frame))
          .await
          .map_err(Error::Library);
        let _ = writer.close().await;
        let _ = tx.send(res);
        break;
      }
    }
  }
}
