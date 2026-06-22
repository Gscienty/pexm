#!/bin/bash
set -e

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
SERVICE_DIR="$ROOT_DIR/service"
PID_FILE="/tmp/pexm-local-debug.pid"

start() {
    echo "==> Starting local debug services..."

    # 1. dal (gRPC :4000)
    echo "  [1/6] Starting dal..."
    cd "$SERVICE_DIR/dal"
    cargo run &>/tmp/pexm-dal.log &
    echo $! >> "$PID_FILE"
    sleep 2

    # 2. clsi (gRPC :5000)
    echo "  [2/6] Starting clsi..."
    cd "$SERVICE_DIR/clsi"
    cargo run &>/tmp/pexm-clsi.log &
    echo $! >> "$PID_FILE"
    sleep 2

    # 3. bff (HTTP :3000, depends on dal)
    echo "  [3/6] Starting bff..."
    cd "$SERVICE_DIR/bff"
    cargo run &>/tmp/pexm-bff.log &
    echo $! >> "$PID_FILE"
    sleep 2

    # 4. real_time (WebSocket :3001)
    echo "  [4/6] Starting real_time..."
    cd "$SERVICE_DIR/real_time"
    cargo run &>/tmp/pexm-realtime.log &
    echo $! >> "$PID_FILE"
    sleep 2

    # 5. web (Astro dev :4321)
    echo "  [5/6] Starting web..."
    cd "$SERVICE_DIR/web"
    pnpm dev &>/tmp/pexm-web.log &
    echo $! >> "$PID_FILE"
    sleep 3

    # 6. nginx
    echo "  [6/6] Starting nginx..."
    NGINX_CONF="$ROOT_DIR/service-ce/nginx/default.conf"
    if [ ! -f "$NGINX_CONF" ]; then
        echo "  ERROR: nginx config not found: $NGINX_CONF"
    elif ! [ -x ~/work/nginx/objs/nginx ]; then
        echo "  ERROR: nginx binary not found: ~/work/nginx/objs/nginx"
    else
        ~/work/nginx/objs/nginx -c "$NGINX_CONF" 2>/tmp/pexm-nginx-start.log || {
            echo "  ERROR: nginx failed to start, see /tmp/pexm-nginx-start.log"
            cat /tmp/pexm-nginx-start.log
        }
    fi

    echo "==> All services started. PIDs saved to $PID_FILE"
    echo "    Logs: /tmp/pexm-*.log"
}

stop() {
    echo "==> Stopping local debug services..."

    # Stop nginx first
    if [ -f /tmp/nginx.pid ]; then
        nginx_pid=$(cat /tmp/nginx.pid)
        kill "$nginx_pid" 2>/dev/null && echo "  Stopped nginx (pid $nginx_pid)" || true
        rm -f /tmp/nginx.pid
    fi

    # Stop all recorded PIDs
    if [ -f "$PID_FILE" ]; then
        while read -r pid; do
            if kill -0 "$pid" 2>/dev/null; then
                kill "$pid" 2>/dev/null && echo "  Stopped pid $pid" || true
            fi
        done < "$PID_FILE"
        rm -f "$PID_FILE"
    fi

    # Fallback: kill by port
    for port in 4000 5000 3000 3001 4321; do
        pid=$(lsof -ti :$port 2>/dev/null || true)
        if [ -n "$pid" ]; then
            kill $pid 2>/dev/null && echo "  Killed process on port $port (pid $pid)" || true
        fi
    done

    echo "==> All services stopped."
}

reload() {
    echo "==> Reloading nginx..."
    if [ -f /tmp/nginx.pid ]; then
        ~/work/nginx/objs/nginx -c "$ROOT_DIR/service-ce/nginx/default.conf" -s reload && echo "  nginx reloaded." || echo "  ERROR: nginx reload failed"
    else
        echo "  ERROR: nginx not running (pid file not found)"
    fi
}

case "${1:-}" in
    start)
        start
        ;;
    stop)
        stop
        ;;
    reload)
        reload
        ;;
    *)
        echo "Usage: $0 {start|stop|reload}"
        exit 1
        ;;
esac
