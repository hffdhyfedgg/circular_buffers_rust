---
id: 001_storage-view-contracts
status: draft
type: architecture
constraints_touched:
  - ARCH-001
  - ARCH-002
  - ARCH-003
  - ARCH-004
  - SAF-001
  - SAF-002
  - SEM-001
  - SEM-002
  - SEM-003
  - STRUCT-001
  - PLAT-001
  - PLAT-002
  - PROC-001
  - PROC-002
  - PROC-003
constraints_added: []
constraints_removed: []
created_by: agent
approved_by: null
---

# Delta Spec: Контракты хранилищ и представлений

## 1. Цель

Зафиксировать контракты хранилищ в `raw-storage` и базовых представлений в `strided-mem`, вынести кольцевую логику и логику с ограничениями по длине («хвосты») из `strided-mem`, а также сформировать модульную структуру трейтов без использования времён жизни в типах представлений.

---

## 2. Контекст

Текущая реализация `strided-mem` содержит монолитные типы кольцевых представлений (`RingView`, `Ring2NView`) и ограничений длины (`TailView`), что нарушает правила композиции и разделения зон ответственности (ARCH-002, ARCH-003). Кроме того, базовые трейты представлений не вынесены в структурированную модуль-систему `traits/` (STRUCT-001), а параметры индексации фиксировали `usize` без явной поддержки типов индексов (`isize` / `usize`).

В этом архитектурном проходе фиксируются контракты хранилищ (`raw-storage`) и фундаментальные трейты представлений (`strided-mem`), поддерживающие свободную композицию с обобщённым типом индекса `Idx` и ассоциированным типом источника `type Source`.

---

## 3. Затронутые ограничения

- `ARCH-001`: Представления не владеют памятью и не содержат времён жизни в типах за счёт связывания через ассоциированный тип `type Source` и параметризации индексом `Idx`.
- `ARCH-002`: Композиция представлений выполняется мономорфизированной цепочкой без `dyn` и без аллокаций.
- `ARCH-003`: `strided-mem` содержит только фундаментальные базовые представления. Кольцевые буферы и закольцованность выносятся на уровень оркестрации.
- `ARCH-004`: Явно зафиксирован механизм связи представления и источника данных через ассоциированный тип `type Source`.
- `SAF-001`, `SAF-002`: `unsafe` локализован в подмодулях реализаций, непересекаемость изменяемых представлений гарантируется конструкторами/разделителями.
- `SEM-001`, `SEM-002`, `SEM-003`: Трейты обобщены над типами элементов `<T>` и индексов `<Idx>`. Кольцевые и обратные операции вынесены в специализированные слои.
- `STRUCT-001`: Описания всех трейтов располагаются строго в `src/traits.rs` (для `raw-storage`) и в подмодулях `src/traits/` (для `strided-mem`).
- `PLAT-001`, `PLAT-002`: Режим `#![no_std]`, отсутствие внешних зависимостей, операции `core::ops`.
- `PROC-001`, `PROC-002`, `PROC-003`: Соблюдение процессов архитектурного прохода и ролевого разделения.

---

## 4. Архитектурное решение

1. **Разделение трейтов хранилищ (`raw-storage`):**
   - Трейты `Storage`, `StorageMut`, `VolatileStorage`, `ResizableStorage` актуализируются в `raw-storage/src/traits.rs`.
   - Вводятся точные Creusot-аннотации (`#[requires]`, `#[ensures]`) и комментарии с перечнем затрагиваемых ограничений.

2. **Модульная структура трейтов представлений (`strided-mem`):**
   - Модуль `strided-mem/src/traits.rs` преобразуется в каталог `strided-mem/src/traits/`.
   - В `strided-mem/src/traits/mod.rs` объявляются базовые фундаментальные контракты: `View<Idx>`, `ViewMut<Idx>`, `ContiguousView<Idx>`, `ContiguousViewMut<Idx>`.
   - В `strided-mem/src/traits/invert.rs` объявляется контракт инвертирующего представления: `InvertibleView<Idx>`.

3. **Обобщённая индексация и отказ от lifetimes в типах представлений:**
   - Все трейты представлений параметризуются типом индекса `Idx` (по умолчанию `usize`, с поддержкой `isize` при композиции).
   - Трейт `View<Idx>` определяет ассоциированные типы `Item` (тип элемента) и `Source` (тип источника данных/хранилища/внутреннего представления).
   - Представления не требуют явного параметра времени жизни `'a` в объявлении самого типа/трейта (ссылка с временем жизни возвращается из методов доступа).

4. **Удаление монолитных кольцевых и хвост-представлений из `strided-mem`:**
   - Файлы `strided-mem/src/ring_view.rs` и `strided-mem/src/tail_view.rs` удаляются (или их типы исключаются из публичного экспорта `strided-mem`), так как кольцевая и tail-индексация не являются фундаментальными свойствами `strided-mem`.

5. **Механизм композиции (ARCH-004):**
   - Источником представления может служить как хранилище `S: Storage`, так и другое представление `V: View<Idx>`. Это выражается через `type Source`.

---

## 5. Файлы для изменения

```yaml
files_to_change:
  - raw-storage/src/traits.rs
  - strided-mem/src/lib.rs
  - strided-mem/src/traits.rs
  - strided-mem/src/traits/mod.rs
  - strided-mem/src/traits/invert.rs
  - strided-mem/src/ring_view.rs
  - strided-mem/src/tail_view.rs

files_forbidden_to_change:
  - ring-buf/src/lib.rs
  - ring-buf/src/cbuf.rs
  - ring-buf/src/cbuf2n.rs
  - ring-buf/src/ops.rs
  - ring-buf/src/traits.rs
```

---

## 6. Целевые сигнатуры

### 6.1. Контракты `raw-storage` (`raw-storage/src/traits.rs`)

```rust
/// Базовый контракт неизменяемого непрерывного хранилища.
/// Затрагиваемые ограничения: ARCH-001, SEM-003, STRUCT-001, PLAT-001.
pub trait Storage {
    /// Тип элементов, хранимых в памяти.
    type Item;

    /// Возвращает количество инициализированных элементов в хранилище.
    // Class: hot-total
    fn len(&self) -> usize;

    /// Возвращает `true`, если хранилище не содержит элементов.
    // Class: hot-total
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Возвращает общую ёмкость выделенной памяти в элементах.
    // Class: cold-checked
    fn capacity(&self) -> usize;

    /// Предоставляет доступ к непрерывному неизменяемому срезу памяти.
    // Class: hot-total
    #[ensures(result.len() == self.len())]
    fn as_slice(&self) -> &[Self::Item];
}

/// Контракт мутабельного непрерывного хранилища.
/// Затрагиваемые ограничения: ARCH-001, SEM-003, STRUCT-001, PLAT-001.
pub trait StorageMut: Storage {
    /// Предоставляет доступ к непрерывному мутабельному срезу памяти.
    // Class: hot-total
    #[ensures(result.len() == self.len())]
    fn as_mut_slice(&mut self) -> &mut [Self::Item];
}

/// Аппаратный контракт для volatile-доступа (MMIO).
/// Затрагиваемые ограничения: ARCH-001, SAF-001, STRUCT-001, PLAT-001.
pub unsafe trait VolatileStorage {
    /// Тип элементов в регистровом пространстве.
    type Item;

    /// Количество регистров/элементов.
    // Class: hot-total
    fn len(&self) -> usize;

    /// Чтение из аппаратного регистра по индексу.
    // Class: hot-unchecked
    #[requires(index < self.len())]
    unsafe fn read_volatile(&self, index: usize) -> Self::Item;

    /// Запись в аппаратный регистр по индексу.
    // Class: hot-unchecked
    #[requires(index < self.len())]
    unsafe fn write_volatile(&mut self, index: usize, value: Self::Item);
}

/// Контракт динамически изменяемого хранилища.
/// Затрагиваемые ограничения: ARCH-001, STRUCT-001, PLAT-001.
#[cfg(feature = "alloc")]
pub trait ResizableStorage: StorageMut {
    /// Изменяет ёмкость хранилища до указанной.
    // Class: cold-checked
    fn try_resize(&mut self, new_capacity: usize) -> Result<(), crate::error::StorageError>;
}
```

### 6.2. Базовые контракты `strided-mem` (`strided-mem/src/traits/mod.rs`)

```rust
pub mod invert;

/// Базовый контракт неизменяемого представления геометрии доступа.
/// Затрагиваемые ограничения: ARCH-001, ARCH-002, ARCH-004, SEM-003, STRUCT-001.
pub trait View<Idx = usize> {
    /// Тип элементов, доступных через представление.
    type Item;

    /// Тип базового источника данных или внутреннего представления.
    type Source;

    /// Количество элементов, доступных через представление.
    // Class: hot-total
    fn len(&self) -> usize;

    /// Проверка на пустоту представления.
    // Class: hot-total
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Безопасный проверяемый доступ к элементу по логическому индексу.
    // Class: cold-checked
    fn try_get(&self, index: Idx) -> Option<&Self::Item>;

    /// Прямой доступ к элементу по логическому индексу.
    // Class: hot-unchecked
    fn get(&self, index: Idx) -> &Self::Item;
}

/// Контракт мутабельного представления геометрии доступа.
/// Затрагиваемые ограничения: ARCH-001, ARCH-002, ARCH-004, SEM-003, STRUCT-001.
pub trait ViewMut<Idx = usize>: View<Idx> {
    /// Безопасный проверяемый мутабельный доступ к элементу.
    // Class: cold-checked
    fn try_get_mut(&mut self, index: Idx) -> Option<&mut Self::Item>;

    /// Прямой мутабельный доступ к элементу по логическому индексу.
    // Class: hot-unchecked
    fn get_mut(&mut self, index: Idx) -> &mut Self::Item;
}

/// Контракт представления, гарантирующего расположение элементов в 1 или 2 непрерывных фрагментах.
/// Затрагиваемые ограничения: ARCH-001, ARCH-002, STRUCT-001.
pub trait ContiguousView<Idx = usize>: View<Idx> {
    /// Возвращает элементы представления в виде пары непрерывных срезов.
    // Class: hot-total
    fn as_slices(&self) -> (&[Self::Item], &[Self::Item]);
}

/// Мутабельный аналог `ContiguousView`.
/// Затрагиваемые ограничения: ARCH-001, ARCH-002, STRUCT-001.
pub trait ContiguousViewMut<Idx = usize>: ViewMut<Idx> + ContiguousView<Idx> {
    /// Возвращает элементы представления в виде пары непрерывных мутабельных срезов.
    // Class: hot-total
    fn as_slices_mut(&mut self) -> (&mut [Self::Item], &mut [Self::Item]);
}
```

### 6.3. Контракт инверсии `strided-mem` (`strided-mem/src/traits/invert.rs`)

```rust
use crate::traits::View;

/// Контракт представления, меняющего порядок доступа к элементам на противоположный.
/// Затрагиваемые ограничения: ARCH-002, SEM-001, STRUCT-001.
pub trait InvertibleView<Idx = usize>: View<Idx> {
    /// Возвращает ссылку на внутреннее инвертируемое представление/источник.
    // Class: setup
    fn inner(&self) -> &Self::Source;
}
```

---

## 7. Алгоритм реализации

1. **Обновление `raw-storage`:**
   - В `raw-storage/src/traits.rs` привести комментарии и атрибуты в соответствие с целевыми сигнатурами раздела 6.1.
   - Проверить, что все типы компилируются без стандартной библиотеки (`#![no_std]`).

2. **Реструктуризация модуля `traits` в `strided-mem`:**
   - Преобразовать `strided-mem/src/traits.rs` в директорию `strided-mem/src/traits/`.
   - Создать `strided-mem/src/traits/mod.rs` и поместить туда контракты `View<Idx>`, `ViewMut<Idx>`, `ContiguousView<Idx>`, `ContiguousViewMut<Idx>` по сигнатурам из 6.2.
   - Создать `strided-mem/src/traits/invert.rs` и поместить туда контракт `InvertibleView<Idx>` по сигнатуре из 6.3.

3. **Очистка `strided-mem` от специфичных буферных представлений:**
   - Удалить или полностью очистить публичный экспорт файлов `strided-mem/src/ring_view.rs` и `strided-mem/src/tail_view.rs`.
   - Обновить `strided-mem/src/lib.rs`, удалив экспорты `RingView`, `Ring2NView`, `TailView` и реэкспортировав подмодуль `traits`.

4. **Проверка компиляции workspace:**
   - Выполнить `cargo check --workspace --no-default-features` для подтверждения целостности сборки `raw-storage` и `strided-mem`.

---

## 8. Требования к безопасности

1. Все контракты `View` и `Storage` строго локализуют `unsafe` в структурах реализации (в `ptr.rs` или конкретных реализациях).
2. Безопасность мутабельного доступа обеспечивается правилами заимствования Rust через ссылки `&mut`.
3. Все предусловия функций с прямым доступом помечены Creusot-аннотациями `#[requires(...)]`.

---

## 9. Требования к горячим путям

1. Функции класса `hot-total` (`len`, `is_empty`, `as_slices`) и `hot-unchecked` (`get`, `get_mut`) не содержат `Option`, `Result`, динамических аллокаций (`alloc`) или виртуальных вызовов (`dyn`).
2. Метод `try_get` относится к классу `cold-checked` и возвращает `Option`.

---

## 10. Тесты

1. Юнит-тесты в `raw-storage/src/traits.rs` (или в модулях имплементаций) для проверки соответствия контрактам `Storage` и `StorageMut`.
2. Юнит-тесты в `strided-mem/src/traits/mod.rs` на базовое поведение проверяемого и не проверяемого доступа `View<usize>` и `View<isize>`.

---

## 11. Критерий готовности

- Код контрактов в `raw-storage` и `strided-mem` полностью соответствует целевым сигнатурам раздела 6.
- Файлы изменены strictly по списку `files_to_change`, файлы из `files_forbidden_to_change` не затрагивались.
- Кольцевые и tail-представления удалены из экспорта `strided-mem`.
- Команда `cargo check --workspace` успешно проходит.

---

## 12. Открытые вопросы

Нет.
