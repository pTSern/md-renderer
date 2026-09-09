# Network Protocol Analysis: User Datagram Protocol (UDP)

The **User Datagram Protocol** (UDP) is defined in [RFC 768](https://www.rfc-editor.org/rfc/rfc768). It provides a connectionless, unreliable datagram service on top of the Internet Protocol (IP).

---

## 1. UDP Datagram Format

```udp
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|          Source Port          |       Destination Port        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|            Length             |           Checksum            |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                             data                              |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

---

## 2. DNS Query / Response Flow over UDP

```mermaid
sequenceDiagram
    autonumber
    actor Browser
    participant Resolver as Local DNS Resolver
    participant RootDNS as Root Nameserver (UDP:53)
    participant AuthDNS as Authoritative DNS (UDP:53)

    Browser->>Resolver: Resolve example.com
    Resolver->>RootDNS: UDP: Who has .com?
    RootDNS-->>Resolver: UDP: Refer to TLD Nameserver
    Resolver->>AuthDNS: UDP: IP for example.com?
    AuthDNS-->>Resolver: UDP: 93.184.216.34
    Resolver-->>Browser: Return IP 93.184.216.34
```

---

## 2.1. System Architecture (Cocos Creator Extension Flowchart)

```mermaid
flowchart TD
    subgraph Host_Project["Host Project (Any Cocos Creator Project)"]
        H1["assets/ (Arbitrary user bundles: B1, B2, ...)"]
        H2[".pts Assets, Scenes, Prefabs across project"]
    end

    subgraph Scanner["lazy-registry.ts (Build-Time / Editor-Time Introspection)"]
        S1["Dynamic Bundle Discovery\nScan all folder .meta with isBundle: true"]
        S2["Dynamic Priority Engine\nsecretPriority = max(projectPriorities) + 5"]
        S3["Dynamic Dependency Crawler\nTrace live roots -> needed assets"]
        S4["Dynamic Bundle Mapper\nMap candidate UUID -> owning bundle"]
    end

    subgraph Artifacts["Generated Extension Artifacts (extensions/pts-asset/assets/_$secret/)"]
        A1["_$secret.meta (Dynamic priority)"]
        A2["_lazy.prefab (Contains only unreferenced assets)"]
        A3["pts-bundle-map.json (uuid -> bundleName)"]
    end

    subgraph Runtime["Json.Register.ts (Generic Runtime Engine)"]
        R1["Resolve UUID on hydration"]
        R2{"Is Asset Loaded in Memory?"}
        R3{"Is Asset in a Loaded Bundle?"}
        R4{"Is Asset in an Unloaded Bundle?"}
        R5["Reactive Deferral + Dynamic Getter\nResolve when bundle loads via pipeline hook"]
    end

    Host_Project --> Scanner
    Scanner --> Artifacts
    Artifacts --> Runtime
    R2 -->|"Yes"| ReturnAsset["Return from assetManager.assets / cache"]
    R3 -->|"Yes"| LoadAny["assetManager.loadAny({ uuid, bundle })"]
    R4 -->|"Yes"| R5
```

---

## 3. High-Performance UDP Socket in Rust

Here is a non-blocking asynchronous UDP server implementation in Rust:

```rust
use std::net::SocketAddr;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080".parse::<SocketAddr>()?;
    let socket = UdpSocket::bind(&addr).await?;
    println!("UDP Server listening on: {}", addr);

    let mut buf = [0u8; 1024];

    loop {
        // Receive datagram and client address
        let (len, client_addr) = socket.recv_from(&mut buf).await?;
        println!("Received {} bytes from {}", len, client_addr);

        // Echo response back over UDP
        let response = b"ACK: Datagram received successfully";
        socket.send_to(response, &client_addr).await?;
    }
}
```

---

## 4. Python UDP Client Script

Here is an example client sending DNS or telemetry packets:

```python
import socket
import struct
import time

def send_udp_packet(host: str, port: int, payload: bytes) -> None:
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.settimeout(2.0)
    
    try:
        start_time = time.time()
        sock.sendto(payload, (host, port))
        data, server = sock.recvfrom(1024)
        elapsed_ms = (time.time() - start_time) * 1000
        print(f"[UDP] Received {len(data)} bytes from {server} in {elapsed_ms:.2f}ms")
    except socket.timeout:
        print("[ERROR] UDP Datagram request timed out!")
    finally:
        sock.close()

if __name__ == "__main__":
    send_udp_packet("127.0.0.1", 8080, b"PING: Test Datagram")
```

---

## 5. Packet Metadata (JSON Payload)

```json
{
  "protocol": "UDP",
  "version": 4,
  "sourcePort": 5353,
  "destinationPort": 53,
  "checksumValid": true,
  "length": 42,
  "flags": {
    "broadcast": false,
    "fragmented": false
  }
}
```

---

## 6. Comparison with TCP Header

```packet
title: Transmission Control Protocol Header (RFC 793)
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|          Source Port          |       Destination Port        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        Sequence Number                        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    Acknowledgment Number                      |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|  Data |           |U|A|P|R|S|F|                               |
| Offset| Reserved  |R|C|S|S|Y|I|            Window             |
|       |           |G|K|H|T|N|N|                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|           Checksum            |         Urgent Pointer        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

> [!TIP]
> Notice how the UDP header is only **8 bytes** (64 bits), whereas the TCP header is at least **20 bytes** (160 bits)! This makes UDP significantly faster for real-time applications like DNS, VoIP, gaming, and HTTP/3 QUIC.

---

## 7. Image Rendering Showcase

MDViewer seamlessly resolves relative paths, absolute local file paths, and web images:

### Relative Path Image
![MDViewer App Logo](./logo.png "MDViewer Logo")

