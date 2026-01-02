#!/bin/bash

PID=$(pgrep -f "sova" | head -1)

if [ -z "$PID" ]; then
    echo "sova not running"
    exit 1
fi

echo "Tracing sova (PID $PID)... Press Ctrl+C to stop"

xctrace record \
    --template "Activity Monitor" \
    --attach "$PID" \
    --output "sova_trace.trace"

open "sova_trace.trace"
