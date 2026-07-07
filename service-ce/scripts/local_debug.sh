#!/bin/bash
set -e

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
SERVICE_DIR="$ROOT_DIR/service"
PID_FILE="/tmp/pexm-local-debug.pid"
NGINX_CONF="$ROOT_DIR/service-ce/nginx/default.conf"
NGINX_BIN="$HOME/work/nginx/objs/nginx"
NGINX_PID_FILE="/tmp/nginx.pid"

SVC_PORT=""
SVC_DIR=""
SVC_CMD=""
SVC_ENV=""
SVC_NAME=""
SVC_NGINX=0

get_service_config() {
    SVC_NGINX=0
    case "$1" in
        dal)
            SVC_NAME=dal; SVC_PORT=4000; SVC_DIR="$SERVICE_DIR/dal"; SVC_CMD="cargo run"; SVC_ENV="" ;;
        clsi)
            SVC_NAME=clsi; SVC_PORT=5001; SVC_DIR="$SERVICE_DIR/clsi"; SVC_CMD="cargo run"; SVC_ENV="" ;;
        real_time|realtime)
            SVC_NAME=real_time; SVC_PORT=3001; SVC_DIR="$SERVICE_DIR/real_time"; SVC_CMD="cargo run"; SVC_ENV="" ;;
        bff)
            SVC_NAME=bff; SVC_PORT=3000; SVC_DIR="$SERVICE_DIR/bff"; SVC_CMD="cargo run"; SVC_ENV="REALTIME_ADDR=http://127.0.0.1:3001" ;;
        web)
            SVC_NAME=web; SVC_PORT=4321; SVC_DIR="$SERVICE_DIR/web"; SVC_CMD="pnpm dev"; SVC_ENV="BFF_ADDR=http://127.0.0.1:3000" ;;
        nginx)
            SVC_NAME=nginx; SVC_PORT=80; SVC_DIR=""; SVC_CMD=""; SVC_ENV=""; SVC_NGINX=1 ;;
        *)
            echo "Unknown module: $1"
            echo "Available modules: dal, clsi, real_time, bff, web, nginx"
            return 1 ;;
    esac
}

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
    if [ ! -f "$NGINX_CONF" ]; then
        echo "  ERROR: nginx config not found: $NGINX_CONF"
    elif ! [ -x "$NGINX_BIN" ]; then
        echo "  ERROR: nginx binary not found: $NGINX_BIN"
    else
        "$NGINX_BIN" -c "$NGINX_CONF" 2>/tmp/pexm-nginx-start.log || {
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
    if [ -f "$NGINX_PID_FILE" ]; then
        nginx_pid=$(cat "$NGINX_PID_FILE")
        kill "$nginx_pid" 2>/dev/null && echo "  Stopped nginx (pid $nginx_pid)" || true
        rm -f "$NGINX_PID_FILE"
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

kill_module() {
    local name="$1" port="$2"
    echo "==> Stopping existing $name..."

    if [ "$SVC_NGINX" -eq 1 ]; then
        if [ -f "$NGINX_PID_FILE" ]; then
            local nginx_pid
            nginx_pid=$(cat "$NGINX_PID_FILE")
            kill "$nginx_pid" 2>/dev/null && echo "  Stopped nginx (pid $nginx_pid)" || true
            rm -f "$NGINX_PID_FILE"
        fi
        "$NGINX_BIN" -s stop 2>/dev/null || true
        for _ in $(seq 1 20); do
            if ! lsof -ti :80 -sTCP:LISTEN &>/dev/null; then break; fi
            sleep 0.3
        done
        return 0
    fi

    local killed_any=0
    local pids_to_kill=""

    if [ -n "$port" ]; then
        local pids
        pids=$(lsof -ti :$port 2>/dev/null || true)
        for pid in $pids; do
            pids_to_kill="$pids_to_kill $pid"
            local ppid
            ppid=$(ps -o ppid= -p "$pid" 2>/dev/null | tr -d ' ' || true)
            if [ -n "$ppid" ] && [ "$ppid" != "1" ] && [ "$ppid" != "0" ]; then
                pids_to_kill="$pids_to_kill $ppid"
            fi
        done
    fi

    for pid in $pids_to_kill; do
        kill "$pid" 2>/dev/null && echo "  Killed $name process (pid $pid)" || true
        killed_any=1
    done

    for _ in $(seq 1 10); do
        if [ -n "$port" ] && lsof -ti :$port -sTCP:LISTEN &>/dev/null; then
            sleep 0.5
        else
            break
        fi
    done

    if [ -n "$port" ]; then
        local remaining
        remaining=$(lsof -ti :$port 2>/dev/null || true)
        if [ -n "$remaining" ]; then
            echo "  Force-killing remaining processes on port $port..."
            kill -9 $remaining 2>/dev/null && echo "  Force-killed pid(s) $remaining" || true
            for _ in $(seq 1 20); do
                if ! lsof -ti :$port -sTCP:LISTEN &>/dev/null; then break; fi
                sleep 0.3
            done
        fi
    fi

    if [ -f "$PID_FILE" ]; then
        local tmpfile
        tmpfile=$(mktemp)
        while read -r pid; do
            if kill -0 "$pid" 2>/dev/null; then
                echo "$pid" >> "$tmpfile"
            fi
        done < "$PID_FILE"
        mv "$tmpfile" "$PID_FILE"
    fi

    if [ $killed_any -eq 0 ]; then
        echo "  No running $name process found."
    fi
}

debug() {
    local module="$1"
    if [ -z "$module" ]; then
        echo "Usage: $0 debug <module>"
        echo "Available modules: dal, clsi, real_time, bff, web, nginx"
        return 1
    fi

    get_service_config "$module"

    echo "==> Debug mode for $SVC_NAME"

    # Kill the existing service
    kill_module "$SVC_NAME" "$SVC_PORT"

    if [ "$SVC_NGINX" -eq 1 ]; then
        echo "==> Starting nginx in foreground (Ctrl+C to stop)..."
        if [ ! -f "$NGINX_CONF" ]; then
            echo "  ERROR: nginx config not found: $NGINX_CONF"
            return 1
        fi
        if [ ! -x "$NGINX_BIN" ]; then
            echo "  ERROR: nginx binary not found: $NGINX_BIN"
            return 1
        fi
        exec "$NGINX_BIN" -c "$NGINX_CONF" -g 'daemon off;'
    fi

    echo "==> Starting $SVC_NAME in foreground (Ctrl+C to stop)..."
    cd "$SVC_DIR"
    if [ -n "$SVC_ENV" ]; then
        exec env $SVC_ENV $SVC_CMD
    else
        exec $SVC_CMD
    fi
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
    debug)
        debug "$2"
        ;;
    *)
        echo "Usage: $0 {start|stop|reload|debug <module>}"
        echo "  debug modules: dal, clsi, real_time, bff, web, nginx"
        exit 1
        ;;
esac
