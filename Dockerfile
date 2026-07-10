FROM rust:1

# Instalar dependencias básicas del sistema si fueran necesarias
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add clippy rustfmt

WORKDIR /usr/src/app

# El código se montará como volumen para desarrollo
