#!/usr/bin/env bash
set -euo pipefail

TYPE="$1"    # delta_spec | arch_report | impl_report | rev_report | trace
SLUG="$2"    # короткое осмысленное имя, например "hot-cold-split"

case "$TYPE" in
  delta_spec)
    DIR="delta_specs"
    TEMPLATE="templates/delta_spec.md"
    ;;
  arch_report)
    DIR="reports/architect"
    TEMPLATE="templates/architect_report.md"
    ;;
  impl_report)
    DIR="reports/implementer"
    TEMPLATE="templates/implementer_report.md"
    ;;
  rev_report)
    DIR="reports/reviewer"
    TEMPLATE="templates/reviewer_report.md"
    ;;
  trace)
    DIR="traces"
    TEMPLATE="templates/trace_entry.md"
    ;;
  *)
    echo "Неизвестный тип: $TYPE" >&2
    exit 1
    ;;
esac

# Определяем следующий номер
MAX=0
for f in "$DIR"/[0-9]*; do
  [ -f "$f" ] || continue
  NUM=$(basename "$f" | grep -oE '^[0-9]+' || true)
  [ -n "$NUM" ] && [ "$NUM" -gt "$MAX" ] && MAX="$NUM"
done
NEXT=$((MAX + 1))
NUM=$(printf "%03d" "$NEXT")

FILENAME="${NUM}_${SLUG}.md"
FILEPATH="${DIR}/${FILENAME}"

if [ -f "$FILEPATH" ]; then
  echo "Файл уже существует: $FILEPATH" >&2
  exit 1
fi

cp "$TEMPLATE" "$FILEPATH"
echo "$FILEPATH"
