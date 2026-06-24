#!/bin/bash
set -e

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
SERVICE_DIR="$ROOT_DIR/service"
PID_FILE="/tmp/pexm-local-debug.pid"

# Wait for a TCP port to be listening. Args: $1=port $2=timeout_secs $3=service_name
wait_for_port() {
    local port=$1 timeout=$2 name=$3 elapsed=0
    while ! lsof -iTCP:$port -sTCP:LISTEN -P -n &>/dev/null; do
        if [ $elapsed -ge $timeout ]; then
            echo "  WARNING: $name (port $port) not ready after ${timeout}s, proceeding anyway"
            return 1
        fi
        sleep 1
        elapsed=$((elapsed + 1))
    done
    echo "  $name (port $port) ready after ${elapsed}s"
    return 0
}

start() {
    echo "==> Starting local debug services..."

    # 1. dal (gRPC :4000)
    echo "  [1/6] Starting dal..."
    cd "$SERVICE_DIR/dal"
    cargo run </dev/null &>/tmp/pexm-dal.log &
    echo $! >> "$PID_FILE"
    wait_for_port 4000 60 dal

    # 2. clsi (gRPC :5001)
    echo "  [2/6] Starting clsi..."
    cd "$SERVICE_DIR/clsi"
    cargo run </dev/null &>/tmp/pexm-clsi.log &
    echo $! >> "$PID_FILE"
    wait_for_port 5001 60 clsi

    # 3. real_time (WebSocket :3001)
    echo "  [3/6] Starting real_time..."
    cd "$SERVICE_DIR/real_time"
    cargo run </dev/null &>/tmp/pexm-realtime.log &
    echo $! >> "$PID_FILE"
    wait_for_port 3001 60 real_time

    # 4. bff (HTTP :3000, depends on dal + real_time)
    echo "  [4/6] Starting bff..."
    cd "$SERVICE_DIR/bff"
    REALTIME_ADDR=http://127.0.0.1:3001 cargo run </dev/null &>/tmp/pexm-bff.log &
    echo $! >> "$PID_FILE"
    wait_for_port 3000 60 bff

    # 5. web (Astro dev :4321)
    echo "  [5/6] Starting web..."
    cd "$SERVICE_DIR/web"
    BFF_ADDR=http://127.0.0.1:3000 pnpm dev </dev/null &>/tmp/pexm-web.log &
    echo $! >> "$PID_FILE"
    wait_for_port 4321 60 web

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
    for port in 4000 5001 3000 3001 4321; do
        pid=$(lsof -ti :$port 2>/dev/null || true)
        if [ -n "$pid" ]; then
            kill $pid 2>/dev/null && echo "  Killed process on port $port (pid $pid)" || true
        fi
    done

    echo "==> All services stopped."
}

reload() {
    echo "==> Reloading all services (stop + start)..."
    stop
    sleep 1
    start
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
