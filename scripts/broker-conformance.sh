#!/usr/bin/env bash
set -Eeuo pipefail

KAFKA_PORT="${KAFKA_PORT:-29092}"
NATS_PORT="${NATS_PORT:-24222}"
RUN_ID="${RUN_ID:-infra-broker-${USER:-agent}-$$}"
KAFKA_CONTAINER="${RUN_ID}-kafka"
NATS_CONTAINER="${RUN_ID}-nats"
FAILED=0

cleanup() {
  if [[ "${FAILED}" == "1" ]]; then
    docker logs "${KAFKA_CONTAINER}" 2>&1 || true
    docker logs "${NATS_CONTAINER}" 2>&1 || true
  fi
  docker rm -f "${KAFKA_CONTAINER}" "${NATS_CONTAINER}" >/dev/null 2>&1 || true
}

on_error() {
  FAILED=1
}

trap on_error ERR
trap cleanup EXIT

wait_for_port() {
  local host="$1"
  local port="$2"
  local label="$3"
  local attempt
  for attempt in $(seq 1 90); do
    if timeout 1 bash -c "</dev/tcp/${host}/${port}" 2>/dev/null; then
      echo "${label} ready at ${host}:${port}"
      return 0
    fi
    sleep 1
  done
  echo "${label} did not become ready at ${host}:${port}" >&2
  return 1
}

docker run --detach --rm \
  --name "${KAFKA_CONTAINER}" \
  --publish "127.0.0.1:${KAFKA_PORT}:9092" \
  --env KAFKA_NODE_ID=1 \
  --env KAFKA_PROCESS_ROLES=broker,controller \
  --env KAFKA_LISTENERS=PLAINTEXT://:9092,CONTROLLER://:9093 \
  --env "KAFKA_ADVERTISED_LISTENERS=PLAINTEXT://127.0.0.1:${KAFKA_PORT}" \
  --env KAFKA_CONTROLLER_LISTENER_NAMES=CONTROLLER \
  --env KAFKA_LISTENER_SECURITY_PROTOCOL_MAP=CONTROLLER:PLAINTEXT,PLAINTEXT:PLAINTEXT \
  --env KAFKA_CONTROLLER_QUORUM_VOTERS=1@localhost:9093 \
  --env KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR=1 \
  --env KAFKA_TRANSACTION_STATE_LOG_REPLICATION_FACTOR=1 \
  --env KAFKA_TRANSACTION_STATE_LOG_MIN_ISR=1 \
  apache/kafka:3.8.1 >/dev/null

docker run --detach --rm \
  --name "${NATS_CONTAINER}" \
  --publish "127.0.0.1:${NATS_PORT}:4222" \
  nats:2.10-alpine -js >/dev/null

wait_for_port 127.0.0.1 "${KAFKA_PORT}" Kafka
wait_for_port 127.0.0.1 "${NATS_PORT}" NATS

FOUNDATIONX_KAFKAX_BROKERS="127.0.0.1:${KAFKA_PORT}" \
  cargo test -p kafkax --test broker_conformance -- --ignored --nocapture --test-threads=1

FOUNDATIONX_NATS_URL="nats://127.0.0.1:${NATS_PORT}" \
  cargo test -p natsx --test broker_conformance -- --ignored --nocapture --test-threads=1

echo "Kafka/NATS broker conformance passed"
