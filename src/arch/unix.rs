use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::io::ErrorKind;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use byteorder::{ByteOrder, LittleEndian};
use crate::error::{UnicomError, UnicomErrorKind};
use crate::node::NodeConfig;
use crate::node::message::request::UnicomRequest;

#[derive(Debug)]
pub enum UnixMessage{
    Error{
        id: u64,
        error: UnicomError,
    },
    Response{
        id: u64,
        data: Vec<u8>
    },
    Request{
        id: u64,
        data: UnicomRequest
    },
    Quit
}

async fn read_head(reader: &mut OwnedReadHalf) -> Result<(u8, u64, usize), UnicomError>{
    let mut buf: [u8; 13] = [0; 13];
    match reader.read(&mut buf).await {
        Ok(13) => Ok((buf[0], LittleEndian::read_u64(&buf[1..9]),LittleEndian::read_u32(&buf[9..13]) as usize)),
        Ok(0) => Err(UnicomError::new(UnicomErrorKind::LostConnection, "lost connection")),
        Ok(_) => Err(UnicomError::new(UnicomErrorKind::DataInvalid, "message head length error")),
        Err(e) => Err(e.into()),
    }
}

async fn read_body(reader: &mut OwnedReadHalf, size: usize) -> Result<Vec<u8>, UnicomError>{
    let mut ret = Vec::new();
    let mut csize = 0;
    loop{
        if size == 0{
            break;
        }
        let mut rsize = size-csize;
        if rsize > 1024{
            rsize = 1024
        }
        
        let mut buf: [u8; 1024] = [0; 1024];
        match reader.read(&mut buf[..rsize]).await {
            Ok(0) => return Err(UnicomError::new(UnicomErrorKind::LostConnection, "lost connection")),
            Ok(n) => {
                csize += n;
                ret.extend_from_slice(&buf[..n]);
                if csize >= size{
                    break;
                }

            },
            Err(e) => {
                if e.kind() != ErrorKind::WouldBlock{
                    return Err(e.into());
                }
            }
        };
        
    } 
    Ok(ret)
}

pub async fn read_init(reader: &mut OwnedReadHalf) -> Result<NodeConfig, UnicomError>{
    let (code, _id, size) = read_head(reader).await?;
    if code != 0x42{
        return Err(UnicomError::new(UnicomErrorKind::ParseError, "Code security not 0x42"))
    }
    Ok(NodeConfig::from_utf8(read_body(reader, size).await?)?)
}

pub async fn read_message(reader: &mut OwnedReadHalf) -> Result<UnixMessage, UnicomError>{
    let (kind, id, size) = read_head(reader).await?;
    match kind {
        0 => Ok(UnixMessage::Error{
            id,
            error: UnicomError::from_utf8(read_body(reader, size).await?)?,
        }),
        1 => Ok(UnixMessage::Request{
            id,
            data: UnicomRequest::from_utf8(read_body(reader, size).await?)?,
        }),
        2 => Ok(UnixMessage::Response{
            id,
            data: read_body(reader, size).await?
        }),
        3 => todo!(),
        4 => Ok(UnixMessage::Quit),
        _ => Err(UnicomError::new(UnicomErrorKind::DataInvalid, "kind message unknown"))
    }
}

async fn write_head(writer: &mut OwnedWriteHalf, kind: u8, id: u64, size: usize)-> Result<(), UnicomError>{
    let mut buf: [u8; 13] = [0; 13];
    buf[0] = kind;
    LittleEndian::write_u64(&mut buf[1..9], id);
    LittleEndian::write_u32(&mut buf[9..13], size as u32);
    writer.write(&buf).await?;
    Ok(())
}

async fn write_body(writer: &mut OwnedWriteHalf, body: &[u8])-> Result<(), UnicomError>{
    let mut current = 0;
    loop{
        if current < body.len(){
            let mut next = current + 1024;
            if next > body.len(){
                next = body.len();
            }
            writer.write(&body[current..next]).await?;
            current = next;
            
        }
        else{
            break;
        }
    }
    Ok(())
}

pub async fn write_init(writer: &mut OwnedWriteHalf, config: &NodeConfig) -> Result<(), UnicomError>{
    let json_data = match serde_json::to_string(&config){
        Ok(data) => data.as_bytes().to_vec(),
        Err(_) => return Err(UnicomError::new(UnicomErrorKind::ParseError, "json init invalid")),
    };
    write_head(writer, 0x42, 0, json_data.len()).await?;
    write_body(writer, &json_data).await?;
    Ok(())

}

pub async fn write_message(writer: &mut OwnedWriteHalf, message: UnixMessage) -> Result<(), UnicomError>{
    match message {
        UnixMessage::Response { id, data} => {
            write_head(writer, 2, id, data.len()).await?;
            write_body(writer, &data).await?;
            Ok(())
        },
        UnixMessage::Request { id, data } => {
            let json_data = match serde_json::to_string(&data){
                Ok(data) => data.as_bytes().to_vec(),
                Err(_) => return Err(UnicomError::new(UnicomErrorKind::ParseError, "json request invalid")),
            };
            write_head(writer, 1, id, json_data.len()).await?;
            write_body(writer, &json_data).await?;
            Ok(())
        },
        UnixMessage::Quit => {
            write_head(writer, 4, 0, 0).await?;
            Ok(())
        },
        UnixMessage::Error { id, error } => {
            let json_data = match serde_json::to_string(&error){
                Ok(data) => data.as_bytes().to_vec(),
                Err(_) => return Err(UnicomError::new(UnicomErrorKind::ParseError, "json Error invalid")),
            };
            write_head(writer, 0, id, json_data.len()).await?;
            write_body(writer, &json_data).await?;
            Ok(())
        },
    }
}