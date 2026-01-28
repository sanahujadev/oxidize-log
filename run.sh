#!/bin/bash

# run.sh - Script para ejecutar scripts predeterminados

SUFFIX=""

# Verifica que se haya pasado un argumento
if [ -z "$1" ]; then
  echo "Uso: $0 [entorno]"
  echo ""
  echo "Entornos disponibles:"
  echo "  dev          - Levanta el servidor de desarrollo con watch"
  echo "  test         - Ejecuta los tests unitarios y de integración"
  echo "  debug        - Ejecuta solo los tests marcados como 'debug'"
  echo "  hash         - Ejecuta solo los tests marcados como 'hash'"
  echo "  staging      - Levanta el servidor de como si fuese el de produccion incluyendo tests automaticos"
  echo "  prod-local   - Levanta el servidor de produccion optimizado para velocidad."
  echo ""
  exit 1
fi

# Selecciona el archivo compose basado en el entorno
if [ "$1" = "dev" ]; then
  SUFFIX="run --example test"
elif [ "$1" = "test" ]; then
  SUFFIX="test"
elif [ "$1" = "ci" ]; then
  SUFFIX=" test && cargo run --example test"
else
  echo "Error: Entorno '$1' no reconocido."
  echo "Entornos disponibles: dev, test, ci"
  exit 1
fi

# Ejecutar docker compose
COMMAND="cargo ${SUFFIX}"
echo "🚀 Ejecutando: ${COMMAND}"
eval "$COMMAND"