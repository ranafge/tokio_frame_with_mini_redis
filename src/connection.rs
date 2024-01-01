use mini_redis::{Frame, Result};
use mini_redis::Frame::Error::Incomplete;
use tokio::net::TcpStream;
use bytes::Buf;
use std::io::Cursor;
use std::io::BufWriter;
use bytes::BytesMut;
use tokio::io::{self, AsyncWriteExt};
use mini_redis::Frame;
enum Frame {
    Simple(String),
    Error(String),
    Integer(u64),
    Bulk(Bytes),
    Null,
    Array(Vec<Frame>),
}
// Connection struct for wrap TcpStream for frame read/write in mini_redis

// struct Connection {
//     stream: TcpStream,
//     // buffer: BytesMut, // mutable version of Bytes
//     buffer: Vec<u8>,
//     cursor: usize,
// }

struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(stream),
            // buffer: BytesMut::with_capacity(4096),
            // buffer: vec![0; 4096],
            // cursor: 0,
            buffer: BytesMut::with_capacity(4096)
        }
    }
    // Read a frame from the connectin.
    ///
    /// Reaturn  `None` if EOF is reached.
    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            // attempt to parse a frame from the buffered data. if enough data has been buffered the frame is retured.
            if let Some(fram) = self.parse_frame()? {
                // first self.parse_frame() is called for parse a redis frmae from self.buffer
                // if there is enough data to parse then read_frame return
                return Ok(Some(frame));
            }
            // There is not enough buffered data to read a frame. attempt to read more data from the socket.
            // On success, the number of bytes is return.  `0` indicated end of the stream.

            // Ensure the buffer has capacity
            if self.buffer.len() == self.cursor {
                // Grow the buffer
                self.buffer.resize(self.cursor * 2, 0); // ????????
            }
            // Read into the buffer, tracing the number of bytes read

            let n = self.stream.read(&mut self.buffer[self.cursor..]).await?;
            if 0 == n {
                if self.cursor == 0 {
                    return Ok(None);
                }else{
                    return Err("connection reset by peer".into());
                }
            }else{
                // Update our cursor
                self.cursor += n
            }

            if 0 == self.stream.read_buff(&mut self.buffer).await? {
                // 0 indicate no more data data receive form peer
                // the remote closed the connection. for this to be a clean shutdown, there should be no data in the read fuffer. if there is
                // this means that the peer closed the socket while sending a frame. otherwise read more data from socket
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    /// write a frame to the connection

    pub async fn write_frame(&mut self, frame: Frame) -> Result<()> {
        // implementation here
    }


    //Tries to parse a frame from the buffer. If the buffer conatins enough
    // data, the frame is returned and the data removed from the buffer. if not enough data has been buffered yet Ok(None) is returned.
    // if buffered data does not represent a valid frame `Err` is returnd.
    fn parse_frame(&mut self) -> Result<Option<Frame>> {
     // Cursor is uesed to track ther current location in the buffer.
     //Cursor also implements Buf from the bytes crate
     // which provides a number of helpful utilities for wroking with bytes

     let mut buf = Cursor::new(&self.buffer[..])   ;
     // The first step is to check if enough data has been burrered to parse a single frame. 
     //this step is usually much faster than doing a full parse of the frame, and allows us to skip allocating data structures
     // to hold the frame data unless we know the full frame has been received
        match Frame::check(&mut buf) {
            Ok(_) => {
                // The check function will have advanced the cursor untill the end of the frame.
                // Since the cursor had position set to zero
                // before Frame::check is called, we obtain the length of the frame by checking the cursor position.
                let len = buf.position() as usize;
                // Reset the position to zero before passing the cursor to // frame::parse
                buf.set_position(0);
                // frame is valid the error is returned
                let frame = Frame::parse(&mut buf)?;
                // Discard the parsed data from the read buffer.
                self.buffer.advance(len);
                // Return the parsed frame to the caller.
                Ok(Some(frame))
            }
            // there is enough data present in the read buffer to parse
            Err(Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }



    }

    async fn write_frame(&mut self, frame: &Frame)->io::Result<()> {
        match frame {
            Frame::Simple(val) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?
            }
            Frame::Error(val) => {
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all("b\r\n").await?;
            }
            Frame::Integer(val) => {
                self.stream.write_u8(b':').await?;
                self.write_decimal(*val).await?
            }
            Frame::Null => {
                self.stream.write_all(b"$-1\r\n").await?;
            }
            Frame::Bulk(val) =>{
                let len = val.len();
                self.stream.write_u8(b'$').await?;
                self.write_decimal(len as u64).await?;
                self.stream.write_all(val).await?;
                self.stream.write_all(b"\r\n").await?;


            }
            Frame::Array(_val) => unimplemented!(),
        }
        self.stream.flush().await;
        Ok(())
    }





}
