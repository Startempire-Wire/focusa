#!/bin/bash
# STRESS TEST: Real load against focusa-daemon
# Tests: concurrent turns, event throughput, memory pressure

BASE_URL="http://127.0.0.1:8787"
CONCURRENT=10
TOTAL_TURNS=100

echo "=== FOCUSA DAEMON STRESS TEST ==="
echo "Concurrent: $CONCURRENT, Total turns: $TOTAL_TURNS"
echo ""

# Function to run a single turn
run_turn() {
    local id=$1
    local turn_id="stress-turn-$id-$(date +%s%N)"
    
    # Start
    curl -s -X POST "$BASE_URL/v1/turn/start" \
        -H "content-type: application/json" \
        -d "{\"turn_id\":\"$turn_id\",\"adapter_id\":\"stress-test\",\"harness_name\":\"stress\",\"timestamp\":\"$(date -Iseconds)\"}" > /dev/null
    
    # Assemble
    curl -s -X POST "$BASE_URL/v1/prompt/assemble" \
        -H "content-type: application/json" \
        -d "{\"turn_id\":\"$turn_id\",\"raw_user_input\":\"Stress test turn $id\",\"format\":\"string\"}" > /dev/null
    
    # Complete
    curl -s -X POST "$BASE_URL/v1/turn/complete" \
        -H "content-type: application/json" \
        -d "{\"turn_id\":\"$turn_id\",\"assistant_output\":\"Stress response for turn $id\",\"artifacts\":[],\"errors\":[]}" > /dev/null
    
    echo "$turn_id"
}

export -f run_turn
export BASE_URL

echo "Test 1: Sequential baseline (10 turns)"
START=$(date +%s.%N)
for i in {1..10}; do
    run_turn $i > /dev/null
done
END=$(date +%s.%N)
SEQ_TIME=$(echo "$END - $START" | bc)
SEQ_RATE=$(echo "scale=2; 10 / $SEQ_TIME" | bc)
echo "  Time: ${SEQ_TIME}s, Rate: ${SEQ_RATE} turns/sec"

echo ""
echo "Test 2: Concurrent stress ($CONCURRENT parallel, $TOTAL_TURNS total)"
START=$(date +%s.%N)
# Use parallel processing
PIDS=()
for i in $(seq 1 $TOTAL_TURNS); do
    run_turn $i > /tmp/turn_$i.log &
    PIDS+=($!)
    
    # Limit concurrent
    if (( i % CONCURRENT == 0 )); then
        wait ${PIDS[@]}
        PIDS=()
    fi
done
wait ${PIDS[@]} 2>/dev/null
END=$(date +%s.%N)

CON_TIME=$(echo "$END - $START" | bc)
CON_RATE=$(echo "scale=2; $TOTAL_TURNS / $CON_TIME" | bc)
echo "  Time: ${CON_TIME}s, Rate: ${CON_RATE} turns/sec"

echo ""
echo "Test 3: Event log verification"
EVENT_COUNT=$(curl -s "$BASE_URL/v1/events/recent?limit=500" | grep -o '"id"' | wc -l)
echo "  Events in log: $EVENT_COUNT"

echo ""
echo "Test 4: Memory check (semantic)"
MEM_COUNT=$(curl -s "$BASE_URL/v1/memory/semantic?limit=500" | grep -o '"key"' | wc -l)
echo "  Semantic records: $MEM_COUNT"

echo ""
echo "Test 5: Focus stack integrity"
STACK=$(curl -s "$BASE_URL/v1/focus/stack")
FRAME_COUNT=$(echo "$STACK" | grep -o '"id"' | wc -l)
echo "  Frames in stack: $FRAME_COUNT"

echo ""
echo "=== STRESS TEST COMPLETE ==="
echo "Sequential rate: ${SEQ_RATE} turns/sec"
echo "Concurrent rate: ${CON_RATE} turns/sec"

if (( $(echo "$CON_RATE > 10" | bc -l) )); then
    echo "✓ Performance acceptable (>10 turns/sec)"
else
    echo "⚠ Performance low (<10 turns/sec)"
fi
