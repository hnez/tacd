export function nbdServe(url: string, file: File) {
  const ws = new WebSocket(url);

  ws.onmessage = (msg) => {
    msg.data.arrayBuffer().then((req: ArrayBuffer) => {
      let req_dv = new DataView(req);

      let magic = req_dv.getUint32(0);
      let cmd_type = req_dv.getUint32(4);
      let cookie = req_dv.getBigUint64(8);
      let offset_big = req_dv.getBigUint64(16)
      let length = req_dv.getUint32(24);

      let end_big = offset_big + BigInt(length);

      if (end_big > BigInt(Number.MAX_SAFE_INTEGER)) {
        console.log("Requested offset too large");
        return        
      }

      let offset = Number(offset_big);
      let end = Number(end_big);
      
      if (magic !== 0x25609513) {
        console.log("Got unexpected NBD Magic:", magic);
        return;
      }

      if (cmd_type !== 0x00000000) {
        console.log("Got unexpected Request Type:", cmd_type);
        return;
      }

      let resp = new ArrayBuffer(16);
      let resp_dv = new DataView(resp);

      resp_dv.setUint32(0, 0x67446698);
      resp_dv.setUint32(4, 0);
      resp_dv.setBigUint64(8, cookie);

      let data = file.slice(offset, end);      

      ws.send(new Blob([resp, data]));

      console.log("sent", data.size, "bytes in response to", length, "byte request at", offset);
    });
  };
}
