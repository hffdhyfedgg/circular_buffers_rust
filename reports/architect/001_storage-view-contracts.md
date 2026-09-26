# Отчёт архитектора: storage-view-contracts

## Метаданные
- Дата: 2026-03-30
- Задача из STATE.md: контракты хранилищ и представлений (скоуп: raw-storage и strided-mem)
- Дельта-спецификация: delta_specs/001_storage-view-contracts.md

## Проанализированные ограничения
| Ограничение | Статус | Конфликт |
|---|---|---|
| ARCH-001 | active | нет |
| ARCH-002 | active | нет |
| ARCH-003 | active | нет |
| ARCH-004 | active | нет |
| SAF-001 | active | нет |
| SAF-002 | active | нет |
| SEM-001 | active | нет |
| SEM-002 | active | нет |
| SEM-003 | active | нет |
| STRUCT-001 | active | нет |
| PLAT-001 | active | нет |
| PLAT-002 | active | нет |
| PROC-001 | active | нет |
| PROC-002 | active | нет |
| PROC-003 | active | нет |

## Архитектурное решение (кратко)
Зафиксированы контракты хранилищ в `raw-storage/src/traits.rs` и фундаментальных представлений в `strided-mem/src/traits/mod.rs` и `strided-mem/src/traits/invert.rs`. Связывание представлений с хранилищами/источниками переведено на ассоциированный тип `type Source` без времён жизни в типах представлений, а доступ параметризован обобщённым типом индекса `Idx`. Из `strided-mem` исключены монолитные кольцевые (`RingView`, `Ring2NView`) и tail-представления (`TailView`), ответственность за которые выносится на оркестратор `ring-buf`.

## Вопросы к человеку
Нет.

## Статус
status: draft
