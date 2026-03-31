#!/usr/bin/env python3
"""
REAL SSE client test - connects to focusa-daemon and verifies streaming.
Tests: connection, event receive, concurrent clients, reconnection.
"""

import requests
import json
import time
import threading
import sys
from datetime import datetime

BASE_URL = "http://127.0.0.1:8787"
SSE_URL = f"{BASE_URL}/v1/events/stream"

class SSEClient:
    """Simple SSE client for testing."""
    
    def __init__(self, client_id):
        self.client_id = client_id
        self.events = []
        self.connected = False
        self.errors = []
        
    def connect(self, duration_sec=5):
        """Connect to SSE stream for specified duration."""
        try:
            headers = {"Accept": "text/event-stream"}
            response = requests.get(SSE_URL, headers=headers, stream=True, timeout=10)
            self.connected = response.status_code == 200
            
            if not self.connected:
                self.errors.append(f"HTTP {response.status_code}")
                return
                
            start_time = time.time()
            buffer = ""
            
            for chunk in response.iter_content(chunk_size=1024, decode_unicode=True):
                if time.time() - start_time > duration_sec:
                    break
                    
                buffer += chunk.decode('utf-8') if isinstance(chunk, bytes) else chunk
                
                # Parse SSE format (simplified)
                lines = buffer.split('\n')
                buffer = lines.pop()  # Keep incomplete line
                
                for line in lines:
                    if line.startswith('data:'):
                        data = line[5:].strip()
                        if data:
                            try:
                                event = json.loads(data)
                                self.events.append({
                                    'time': datetime.now().isoformat(),
                                    'data': event
                                })
                            except json.JSONDecodeError:
                                self.events.append({
                                    'time': datetime.now().isoformat(),
                                    'raw': data
                                })
                                
        except Exception as e:
            self.errors.append(str(e))
            self.connected = False

def test_single_client():
    """Test 1: Single client connection."""
    print("Test 1: Single SSE client connection")
    client = SSEClient("single")
    client.connect(duration_sec=3)
    
    if client.connected:
        print(f"  ✓ Connected successfully")
        print(f"  ✓ Received {len(client.events)} events in 3 seconds")
        if client.events:
            print(f"  ✓ Sample event type: {client.events[0]['data'].get('type', 'unknown')}")
        return True
    else:
        print(f"  ✗ Connection failed: {client.errors}")
        return False

def test_concurrent_clients():
    """Test 2: Concurrent clients (stress test)."""
    print("\nTest 2: Concurrent SSE clients (10 parallel)")
    
    clients = []
    threads = []
    
    def run_client(client_id):
        client = SSEClient(f"client-{client_id}")
        client.connect(duration_sec=5)
        clients.append(client)
    
    # Start 10 concurrent clients
    start_time = time.time()
    for i in range(10):
        t = threading.Thread(target=run_client, args=(i,))
        t.start()
        threads.append(t)
    
    for t in threads:
        t.join()
    
    duration = time.time() - start_time
    
    connected = sum(1 for c in clients if c.connected)
    total_events = sum(len(c.events) for c in clients)
    
    print(f"  ✓ {connected}/10 clients connected")
    print(f"  ✓ {total_events} total events received")
    print(f"  ✓ Completed in {duration:.2f}s")
    
    return connected == 10

def test_event_types():
    """Test 3: Verify event types in stream."""
    print("\nTest 3: Event type verification")
    
    client = SSEClient("types")
    client.connect(duration_sec=5)
    
    if not client.events:
        print("  ✗ No events received")
        return False
    
    # Collect unique event types
    types = set()
    for event in client.events:
        if 'data' in event and isinstance(event['data'], dict):
            types.add(event['data'].get('type', 'unknown'))
    
    print(f"  ✓ Received {len(types)} unique event types:")
    for t in sorted(types)[:5]:  # Show first 5
        count = sum(1 for e in client.events if e.get('data', {}).get('type') == t)
        print(f"    - {t}: {count} events")
    
    return len(types) > 0

def test_reconnection():
    """Test 4: Client reconnection."""
    print("\nTest 4: Reconnection resilience")
    
    client = SSEClient("reconnect")
    
    # First connection
    client.connect(duration_sec=2)
    first_count = len(client.events)
    print(f"  ✓ First connection: {first_count} events")
    
    # Reconnect
    client.events = []
    client.connect(duration_sec=2)
    second_count = len(client.events)
    print(f"  ✓ Reconnection: {second_count} events")
    
    if client.connected and second_count > 0:
        print(f"  ✓ Reconnection successful, stream resumed")
        return True
    else:
        print(f"  ✗ Reconnection failed")
        return False

def main():
    print("=" * 60)
    print("FOCUSA SSE STREAMING - REAL CLIENT TEST")
    print("=" * 60)
    print()
    
    # Check daemon health first
    try:
        health = requests.get(f"{BASE_URL}/v1/health", timeout=2).json()
        if not health.get('ok'):
            print("✗ Daemon not healthy")
            return 1
        print(f"✓ Daemon healthy (uptime: {health.get('uptime_ms', 0)}ms)")
        print()
    except Exception as e:
        print(f"✗ Cannot connect to daemon: {e}")
        return 1
    
    # Run tests
    results = []
    
    results.append(("Single Client", test_single_client()))
    results.append(("Concurrent Clients", test_concurrent_clients()))
    results.append(("Event Types", test_event_types()))
    results.append(("Reconnection", test_reconnection()))
    
    # Summary
    print("\n" + "=" * 60)
    print("TEST SUMMARY")
    print("=" * 60)
    
    passed = sum(1 for _, r in results if r)
    total = len(results)
    
    for name, result in results:
        status = "✓ PASS" if result else "✗ FAIL"
        print(f"{status}: {name}")
    
    print()
    print(f"Results: {passed}/{total} tests passed")
    
    if passed == total:
        print("\n✓ SSE STREAMING FULLY OPERATIONAL")
        return 0
    else:
        print("\n✗ SOME TESTS FAILED")
        return 1

if __name__ == "__main__":
    sys.exit(main())
