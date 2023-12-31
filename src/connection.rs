use mini_redis::{Frame, Result};
use tokio::net::TcpStream;

// Connection struct for wrap TcpStream for frame read/write in mini_redis

struct Connection {
    stream: TcpStream,
    // buffer: BytesMut, // mutable version of Bytes
    buffer: Vec<u8>,
    cursor: usize,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
            // buffer: BytesMut::with_capacity(4096),
            buffer: vec![0; 4096],
            cursor: 0,
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
}
