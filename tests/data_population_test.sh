#!/bin/bash
# REAL test: Populate autonomy, ECS, procedural memory, training data
# Runs comprehensive turn sequence to generate data for all systems

BASE_URL="http://127.0.0.1:8787"

echo "=== DATA POPULATION TEST ==="
echo "Tests: Autonomy scoring, ECS handles, Procedural memory, Training export"
echo ""

# Baseline measurements
echo "Baseline measurements:"
AUTONOMY_BEFORE=$(curl -s "$BASE_URL/v1/autonomy")
ARI_BEFORE=$(echo "$AUTONOMY_BEFORE" | grep -o '"ari_score":[0-9.]*' | cut -d: -f2)
echo "  ARI score before: ${ARI_BEFORE:-0.0}"

ECS_BEFORE=$(curl -s "$BASE_URL/v1/ecs/handles" | grep -o '"count":[0-9]*' | cut -d: -f2)
echo "  ECS handles before: ${ECS_BEFORE:-0}"

PROC_BEFORE=$(curl -s "$BASE_URL/v1/memory/procedural" | grep -o '"rule_id"' | wc -l)
echo "  Procedural rules before: ${PROC_BEFORE:-0}"

TRAINING_BEFORE=$(curl -s "$BASE_URL/v1/training/status" | grep -o '"queue_size":[0-9]*' | cut -d: -f2)
echo "  Training queue before: ${TRAINING_BEFORE:-0}"

echo ""
echo "Running 20 turns with varying patterns..."

for i in {1..20}; do
    TURN_ID="data-pop-$i-$(date +%s)"
    
    # Start turn
    curl -s -X POST "$BASE_URL/v1/turn/start" \
        -H "content-type: application/json" \
        -d "{\"turn_id\":\"$TURN_ID\",\"adapter_id\":\"data-pop-test\",\"harness_name\":\"test\",\"timestamp\":\"$(date -Iseconds)\"}" > /dev/null
    
    # Vary the output to create different patterns
    if [ $((i % 4)) -eq 0 ]; then
        # Error pattern
        OUTPUT="Error: Failed to process request"
        ERRORS='["processing_error"]'
    elif [ $((i % 4)) -eq 1 ]; then
        # Success with artifact reference (large content for ECS)
        OUTPUT=$(python3 -c "print('A' * 1000)")
        ERRORS="[]"
    elif [ $((i % 4)) -eq 2 ]; then
        # Pattern for procedural learning
        OUTPUT="Decision made: Use async pattern. Constraint: Must handle timeout."
        ERRORS="[]"
    else
        # Normal success
        OUTPUT="Completed task $i successfully with focus on Agent Wiki Planning"
        ERRORS="[]"
    fi
    
    # Complete turn
    curl -s -X POST "$BASE_URL/v1/turn/complete" \
        -H "content-type: application/json" \
        -d "{\"turn_id\":\"$TURN_ID\",\"assistant_output\":\"$OUTPUT\",\"artifacts\":[],\"errors\":$ERRORS}" > /dev/null
    
    # Reinforce a rule every 5 turns
    if [ $((i % 5)) -eq 0 ]; then
        curl -s -X POST "$BASE_URL/v1/memory/reinforce" \
            -H "content-type: application/json" \
            -d "{\"rule_id\":\"rule-$i\"}" > /dev/null
    fi
done

sleep 1

echo ""
echo "Post-test measurements:"
AUTONOMY_AFTER=$(curl -s "$BASE_URL/v1/autonomy")
ARI_AFTER=$(echo "$AUTONOMY_AFTER" | grep -o '"ari_score":[0-9.]*' | cut -d: -f2)
echo "  ARI score after: ${ARI_AFTER:-0.0}"

ECS_AFTER=$(curl -s "$BASE_URL/v1/ecs/handles" | grep -o '"count":[0-9]*' | cut -d: -f2)
echo "  ECS handles after: ${ECS_AFTER:-0}"

PROC_AFTER=$(curl -s "$BASE_URL/v1/memory/procedural" | grep -o '"rule_id"' | wc -l)
echo "  Procedural rules after: ${PROC_AFTER:-0}"

TRAINING_AFTER=$(curl -s "$BASE_URL/v1/training/status" | grep -o '"queue_size":[0-9]*' | cut -d: -f2)
echo "  Training queue after: ${TRAINING_AFTER:-0}"

echo ""
echo "=== RESULTS ==="
if [ "${ARI_AFTER:-0.0}" != "0.0" ] && [ "${ARI_AFTER:-0.0}" != "$ARI_BEFORE" ]; then
    echo "✓ Autonomy scoring: ARI increased"
else
    echo "✗ Autonomy scoring: ARI still 0.0 (focusa-od0)"
fi

if [ "${ECS_AFTER:-0}" -gt "${ECS_BEFORE:-0}" ]; then
    echo "✓ ECS handles: Created ${ECS_AFTER:-0} handles"
else
    echo "✗ ECS handles: Still empty (focusa-8jz)"
fi

if [ "${PROC_AFTER:-0}" -gt "${PROC_BEFORE:-0}" ]; then
    echo "✓ Procedural memory: ${PROC_AFTER:-0} rules"
else
    echo "✗ Procedural memory: Still empty (focusa-71v)"
fi

if [ "${TRAINING_AFTER:-0}" -gt "${TRAINING_BEFORE:-0}" ]; then
    echo "✓ Training export: ${TRAINING_AFTER:-0} contributions"
else
    echo "✗ Training export: Still empty (focusa-zup)"
fi
