#!/usr/bin/env python3
"""
Simple test client for LostLove Server (Phase 1)

This script tests basic connectivity and handshake with the server.
"""

import socket
import struct
import json
import secrets
import time

# Protocol constants
PROTOCOL_ID = 0x4C4C  # "LL" (LostLove)
PACKET_TYPE_HANDSHAKE_INIT = 0x03
PACKET_TYPE_HANDSHAKE_RESPONSE = 0x04
PACKET_TYPE_DATA = 0x01
PACKET_TYPE_KEEPALIVE = 0x05
PACKET_TYPE_DISCONNECT = 0x06

HEADER_SIZE = 24

def calculate_crc16(data):
    """Calculate CRC16-CCITT checksum"""
    crc = 0xFFFF
    for byte in data:
        crc ^= byte << 8
        for _ in range(8):
            if crc & 0x8000:
                crc = (crc << 1) ^ 0x1021
            else:
                crc <<= 1
            crc &= 0xFFFF
    return crc

def create_packet(packet_type, payload):
    """Create LLP packet"""
    stream_id = 0
    sequence_number = 0
    timestamp = int(time.time() * 1000)
    flags = 0

    # Create header without checksum
    header = struct.pack(
        '>HBHQQQB',
        PROTOCOL_ID,
        packet_type,
        stream_id,
        sequence_number,
        timestamp,
        flags
    )

    # Calculate checksum
    checksum = calculate_crc16(header + payload)

    # Create complete header with checksum
    header_with_checksum = struct.pack(
        '>HBHQQBH',
        PROTOCOL_ID,
        packet_type,
        stream_id,
        sequence_number,
        timestamp,
        flags,
        checksum
    )

    return header_with_checksum + payload

def parse_packet(data):
    """Parse LLP packet"""
    if len(data) < HEADER_SIZE:
        raise ValueError(f"Packet too short: {len(data)} bytes")

    header = struct.unpack('>HBHQQBH', data[:HEADER_SIZE])

    protocol_id = header[0]
    packet_type = header[1]
    stream_id = header[2]
    sequence_number = header[3]
    timestamp = header[4]
    flags = header[5]
    checksum = header[6]

    payload = data[HEADER_SIZE:]

    return {
        'protocol_id': protocol_id,
        'packet_type': packet_type,
        'stream_id': stream_id,
        'sequence_number': sequence_number,
        'timestamp': timestamp,
        'flags': flags,
        'checksum': checksum,
        'payload': payload
    }

def perform_handshake(sock):
    """Perform handshake with server"""
    print("[*] Starting handshake...")

    # Create ClientHello message
    client_random = secrets.token_bytes(32)
    client_hello = {
        'ClientHello': {
            'client_random': list(client_random),
            'protocol_version': 1
        }
    }

    # Serialize to JSON
    payload = json.dumps(client_hello).encode('utf-8')

    # Create packet
    packet = create_packet(PACKET_TYPE_HANDSHAKE_INIT, payload)

    # Send ClientHello
    print(f"[→] Sending ClientHello ({len(packet)} bytes)")
    sock.sendall(packet)

    # Receive ServerHello
    print("[←] Waiting for ServerHello...")
    data = sock.recv(4096)

    if not data:
        raise Exception("Connection closed by server")

    # Parse packet
    response = parse_packet(data)

    if response['packet_type'] != PACKET_TYPE_HANDSHAKE_RESPONSE:
        raise Exception(f"Unexpected packet type: {response['packet_type']}")

    # Parse ServerHello
    server_hello = json.loads(response['payload'])

    print(f"[✓] ServerHello received!")
    print(f"    Session ID: {server_hello['ServerHello']['session_id']}")

    return server_hello['ServerHello']['session_id']

def send_keepalive(sock):
    """Send keepalive packet"""
    packet = create_packet(PACKET_TYPE_KEEPALIVE, b'')
    print("[→] Sending keepalive")
    sock.sendall(packet)

    # Receive response
    data = sock.recv(4096)
    response = parse_packet(data)

    if response['packet_type'] == PACKET_TYPE_KEEPALIVE:
        print("[✓] Keepalive response received")
        return True

    return False

def send_disconnect(sock):
    """Send disconnect packet"""
    packet = create_packet(PACKET_TYPE_DISCONNECT, b'')
    print("[→] Sending disconnect")
    sock.sendall(packet)

def main():
    import argparse

    parser = argparse.ArgumentParser(description='LostLove Test Client')
    parser.add_argument('--host', default='127.0.0.1', help='Server host')
    parser.add_argument('--port', type=int, default=8443, help='Server port')
    parser.add_argument('--keepalive', type=int, default=3, help='Number of keepalive tests')

    args = parser.parse_args()

    print(f"[*] LostLove Test Client")
    print(f"[*] Connecting to {args.host}:{args.port}...")

    try:
        # Connect to server
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.connect((args.host, args.port))

        print("[✓] Connected!")

        # Perform handshake
        session_id = perform_handshake(sock)

        # Send keepalive packets
        print(f"\n[*] Testing with {args.keepalive} keepalive packets...")
        for i in range(args.keepalive):
            time.sleep(1)
            if not send_keepalive(sock):
                print("[!] Keepalive failed")
                break

        # Disconnect
        print("\n[*] Disconnecting...")
        send_disconnect(sock)

        sock.close()
        print("[✓] Test completed successfully!")

    except Exception as e:
        print(f"[✗] Error: {e}")
        import traceback
        traceback.print_exc()
        return 1

    return 0

if __name__ == '__main__':
    exit(main())
