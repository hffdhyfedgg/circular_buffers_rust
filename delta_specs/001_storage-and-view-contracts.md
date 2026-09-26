---
id: 2025-02-18-storage-and-view-contracts
status: draft
type: contract
constraints_touched:
  - ARCH-001
  - ARCH-002
  - ARCH-003
  - PLAT-001
  - PLAT-002
  - SAF-001
  - SAF-002
  - SEM-001
  - SEM-002
  - SEM-003
  - STRUCT-001
constraints_added: []
constraints_removed: []
created_by: agent
approved_by: null
---

# Delta Spec: Контракты хранилищ и представлений (Проход 1)

## 1. Цель

Зафиксировать систему контрактов-трейтов для хранилищ памяти (`raw-storage`) и представлений (`strided-mem`) с аннотациями Creusot в рамках первого архитектурного прохода.

---

## 2. Контекст

Текущая реализация системы содержит монолитную структуру `ring-buf`, а представления в `strided-mem` не выделены в единую систему контрактов, запрещающую владение и обеспечивающую бесплатную композицию (построение представления поверх другого представления).

Данная дельта-спецификация фиксирует фундаментальные контракты для хранилищ (`raw-storage`) и представлений (`strided-mem`). Крейт `ring-buf` сознательно вынесен из первого прохода и будет перестроен в последующем проходе как оркестратор над хранилищем и композицией представлений.

---

## 3. Затронутые ограничения

- **ARCH-001** — Представления не владеют памятью.
- **ARCH-002** — Композиция представлений должна быть нулевой стоимостью (бесплатной абстракцией).
- **ARCH-003** — Кольцевой буфер как оркестратор над Storage и View.
- **PLAT-001** — `#![no_std]` по умолчанию.
- **PLAT-002** — Использование арифметики и операций только из `core`.
- **SAF-001** — `unsafe` локализован в конкретных представлениях и `VolatileStorage`, запрещён в интерфейсах оркестратора.
- **SAF-002** — Гарантия непересечения изменяемых представлений.
- **SEM-001** — Обратная нумерация элементов (индекс 0 — самый новый элемент).
- **SEM-002** — Закольцованность и тотальность доступа без Option/Result/паник в горячих путях.
- **SEM-003** — Все трейты generic по типу элементов или содержат ассоциированный тип `Item`.
- **STRUCT-001** — Все определения трейтов находятся исключительно в модулях `src/traits.rs`.

---

## 4. Архитектурное решение

Разработка разделена на независимые архитектурные проходы:

1. **Проход 1 (Текущий)**:
   - Зафиксировать трейты-контракты в `raw-storage/src/traits.rs` (`Storage`, `StorageMut`, `VolatileStorage`, `ResizableStorage`).
   - Зафиксировать трейты-контракты в `strided-mem/src/traits.rs` (`View`, `ViewMut`, `ContiguousView`, `ContiguousViewMut`, `ViewCompose`, `ViewComposeMut`, `StridedAccess`, `ResizableView`, `MultiChannelView`, `MultiChannelViewMut`).
   - Добавить аннотации Creusot (`#[requires]`, `#[ensures]`) для формальной верификации предусловий и постусловий.

2. **Проход 2 (Последующий)**:
   - Реализовать типы представлений в `strided-mem` (`StridedView`, `RingView`, `InvertView`, `TailView`, `StackView`), удовлетворяющие трейтам из Прохода 1 и поддерживающие произвольную вложенность.

3. **Проход 3 (Финальный)**:
   - Перестроить `ring-buf`, сделав `CBuf<T, S, V>` тонким оркестратором над `S: Storage` и `V: View`.

---

## 5. Файлы для изменения

```yaml
files_to_change:
  - raw-storage/src/traits.rs
  - strided-mem/src/traits.rs

files_forbidden_to_change:
  - ring-buf/src/*
  - constraints/*
```

---

## 6. Целевые сигнатуры

### 6.1. `raw-storage/src/traits.rs`

```rust
/// Базовый контракт неизменяемого непрерывного хранилища памяти.
/// Затрагиваемые ограничения: ARCH-001, PLAT-001, SEM-003, STRUCT-001.
pub trait Storage {
    /// Тип элементов, хранящихся в памяти.
    type Item;

    /// Возвращает количество инициализированных элементов.
    #[requires(true)]
    #[ensures(result <= self.capacity())]
    fn len(&self) -> usize;

    /// Возвращает `true`, если хранилище не содержит элементов.
    #[requires(true)]
    #[ensures(result == (self.len() == 0))]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Возвращает полную ёмкость выделенной памяти в элементах.
    #[requires(true)]
    #[ensures(result >= self.len())]
    fn capacity(&self) -> usize;

    /// Предоставляет доступ к непрерывному неизменяемому срезу памяти.
    #[requires(true)]
    #[ensures(result.len() == self.len())]
    fn as_slice(&self) -> &[Self::Item];
}

/// Контракт мутабельного непрерывного хранилища памяти.
/// Затрагиваемые ограничения: ARCH-001, PLAT-001, SEM-003, STRUCT-001.
pub trait StorageMut: Storage {
    /// Предоставляет доступ к непрерывному мутабельному срезу памяти.
    #[requires(true)]
    #[ensures(result.len() == self.len())]
    fn as_mut_slice(&mut self) -> &mut [Self::Item];
}

/// Контракт для прямого доступа к аппаратным регистрам MMIO.
/// Затрагиваемые ограничения: ARCH-001, SAF-001, STRUCT-001.
///
/// # Safety
/// Реализации выполняют volatile-чтение и запись без создания `&mut T`.
pub unsafe trait VolatileStorage {
    /// Тип элементов в регистре.
    type Item;

    /// Возвращает количество доступных регистров/элементов.
    #[requires(true)]
    #[ensures(true)]
    fn len(&self) -> usize;

    /// Выполняет volatile-чтение из регистра по индексу.
    #[requires(index < self.len())]
    #[ensures(true)]
    unsafe fn read_volatile(&self, index: usize) -> Self::Item;

    /// Выполняет volatile-запись в регистр по индексу.
    #[requires(index < self.len())]
    #[ensures(true)]
    unsafe fn write_volatile(&mut self, index: usize, value: Self::Item);
}

/// Контракт для динамически изменяемого хранилища памяти.
/// Затрагиваемые ограничения: ARCH-001, PLAT-001, STRUCT-001.
#[cfg(feature = "alloc")]
pub trait ResizableStorage: StorageMut {
    /// Изменяет ёмкость хранилища.
    #[requires(true)]
    #[ensures(true)]
    fn try_resize(&mut self, new_capacity: usize) -> core::result::Result<(), crate::error::StorageError>;
}
```

### 6.2. `strided-mem/src/traits.rs`

```rust
/// Базовый контракт неизменяемого представления над элементами типа `Item`.
/// Представления задают геометрию доступа, не владея памятью.
/// Затрагиваемые ограничения: ARCH-001, ARCH-002, SEM-001, SEM-002, SEM-003, STRUCT-001.
pub trait View<'a> {
    /// Тип элементов, доступных через данный view.
    type Item: 'a;

    /// Возвращает логическое количество элементов, доступных через представление.
    #[requires(true)]
    #[ensures(true)]
    fn len(&self) -> usize;

    /// Возвращает `true`, если длина представления равна 0.
    #[requires(true)]
    #[ensures(result == (self.len() == 0))]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Безопасный доступ к элементу по логическому индексу (с проверкой границ).
    #[requires(true)]
    #[ensures(match result { Some(_) => index < self.len(), None => index >= self.len() })]
    fn get(&self, index: usize) -> Option<&'a Self::Item>;

    /// Относительный доступ по знаковому индексу (`rel >= 0` от головы/нового, `rel < 0` от хвоста/старого).
    #[requires(true)]
    #[ensures(true)]
    fn get_rel(&self, rel: isize) -> Option<&'a Self::Item>;

    /// Закольцованный доступ с автоматическим приведением индекса по модулю.
    /// Тотальная функция для непустых представлений (класс: hot-total).
    #[requires(self.len() > 0)]
    #[ensures(true)]
    fn get_wrapping(&self, rel: isize) -> &'a Self::Item;
}

/// Контракт мутабельного представления.
/// Затрагиваемые ограничения: ARCH-001, ARCH-002, SAF-002, SEM-001, SEM-002, SEM-003, STRUCT-001.
pub trait ViewMut<'a>: View<'a> {
    /// Мутабельный доступ к элементу по логическому индексу с проверкой границ.
    #[requires(true)]
    #[ensures(match result { Some(_) => index < self.len(), None => index >= self.len() })]
    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Item>;

    /// Мутабельный относительный доступ по знаковому индексу.
    #[requires(true)]
    #[ensures(true)]
    fn get_rel_mut(&mut self, rel: isize) -> Option<&mut Self::Item>;

    /// Мутабельный закольцованный доступ с приведением индекса по модулю (класс: hot-total).
    #[requires(self.len() > 0)]
    #[ensures(true)]
    fn get_wrapping_mut(&mut self, rel: isize) -> &mut Self::Item;
}

/// Представление, данные которого лежат в не более чем двух непрерывных срезах памяти.
/// Затрагиваемые ограничения: ARCH-001, PLAT-001, STRUCT-001.
pub trait ContiguousView<'a>: View<'a> {
    /// Возвращает доступные элементы как пару непрерывных срезов.
    #[requires(true)]
    #[ensures(result.0.len() + result.1.len() == self.len())]
    fn as_slices(&self) -> (&'a [Self::Item], &'a [Self::Item]);
}

/// Мутабельный аналог `ContiguousView`.
/// Затрагиваемые ограничения: ARCH-001, SAF-002, STRUCT-001.
pub trait ContiguousViewMut<'a>: ViewMut<'a> + ContiguousView<'a> {
    /// Возвращает доступные элементы как пару мутабельных непрерывных срезов.
    #[requires(true)]
    #[ensures(result.0.len() + result.1.len() == self.len())]
    fn as_slices_mut(&mut self) -> (&'a mut [Self::Item], &'a mut [Self::Item]);
}

/// Контракт композиции представлений (обёртка над внутренним источником/представлением).
/// Затрагиваемые ограничения: ARCH-002, STRUCT-001.
pub trait ViewCompose<'a>: View<'a> {
    /// Тип внутреннего источника данных или вложенного представления.
    type Source: View<'a, Item = Self::Item>;

    /// Возвращает ссылку на внутренний источник.
    #[requires(true)]
    #[ensures(true)]
    fn source(&self) -> &Self::Source;
}

/// Мутабельный аналог `ViewCompose`.
/// Затрагиваемые ограничения: ARCH-002, SAF-002, STRUCT-001.
pub trait ViewComposeMut<'a>: ViewCompose<'a> + ViewMut<'a>
where
    Self::Source: ViewMut<'a, Item = Self::Item>,
{
    /// Возвращает мутабельную ссылку на внутренний источник.
    #[requires(true)]
    #[ensures(true)]
    fn source_mut(&mut self) -> &mut Self::Source;
}

/// Контракт для представлений с фиксированным шагом (stride) между элементами.
/// Затрагиваемые ограничения: ARCH-001, STRUCT-001.
pub trait StridedAccess {
    /// Возвращает шаг (stride) между соседними логическими элементами в памяти.
    #[requires(true)]
    #[ensures(result > 0)]
    fn stride(&self) -> usize;
}

/// Контракт для представлений с динамически изменяемой эффективной длиной (хвосты).
/// Затрагиваемые ограничения: ARCH-001, STRUCT-001.
pub trait ResizableView<'a>: View<'a> {
    /// Возвращает текущую видимую (эффективную) длину представления.
    #[requires(true)]
    #[ensures(result <= self.capacity())]
    fn effective_len(&self) -> usize;

    /// Возвращает максимальную физическую ёмкость.
    #[requires(true)]
    #[ensures(result >= self.effective_len())]
    fn capacity(&self) -> usize;

    /// Изменяет видимую длину представления.
    #[requires(new_len <= self.capacity())]
    #[ensures(true)]
    fn set_effective_len(&mut self, new_len: usize) -> core::result::Result<(), crate::error::StridedError>;
}

/// Контракт многоканального представления (стеки буферов).
/// Затрагиваемые ограничения: ARCH-001, ARCH-002, SEM-003, STRUCT-001.
pub trait MultiChannelView<'a> {
    /// Тип элементов в каналах.
    type Item: 'a;

    /// Возвращает количество каналов.
    #[requires(true)]
    #[ensures(true)]
    fn channels(&self) -> usize;

    /// Возвращает длину одного канала в элементах.
    #[requires(true)]
    #[ensures(true)]
    fn channel_len(&self) -> usize;

    /// Возвращает ссылку на элемент канала `channel` по логическому индексу `index`.
    #[requires(channel < self.channels())]
    #[ensures(match result { Some(_) => index < self.channel_len(), None => index >= self.channel_len() })]
    fn get_channel(&self, channel: usize, index: usize) -> Option<&'a Self::Item>;

    /// Закольцованный доступ к элементу канала `channel` по относительному индексу `rel` (класс: hot-total).
    #[requires(channel < self.channels() && self.channel_len() > 0)]
    #[ensures(true)]
    fn get_channel_wrapping(&self, channel: usize, rel: isize) -> &'a Self::Item;
}

/// Мутабельный аналог `MultiChannelView`.
/// Затрагиваемые ограничения: ARCH-001, ARCH-002, SAF-002, STRUCT-001.
pub trait MultiChannelViewMut<'a>: MultiChannelView<'a> {
    /// Мутабельный доступ к элементу канала `channel` по логическому индексу `index`.
    #[requires(channel < self.channels())]
    #[ensures(match result { Some(_) => index < self.channel_len(), None => index >= self.channel_len() })]
    fn get_channel_mut(&mut self, channel: usize, index: usize) -> Option<&mut Self::Item>;

    /// Мутабельный закольцованный доступ к элементу канала `channel` (класс: hot-total).
    #[requires(channel < self.channels() && self.channel_len() > 0)]
    #[ensures(true)]
    fn get_channel_wrapping_mut(&mut self, channel: usize, rel: isize) -> &mut Self::Item;
}
```

---

## 7. Алгоритм реализации

1. В файле `raw-storage/src/traits.rs` обновить трейты `Storage`, `StorageMut`, `VolatileStorage`, `ResizableStorage` в соответствии с указанными выше аннотациями Creusot и атрибутами.
2. В файле `strided-mem/src/traits.rs` заменить текущие определения трейтов на целевые сигнатуры (`View`, `ViewMut`, `ContiguousView`, `ContiguousViewMut`, `ViewCompose`, `ViewComposeMut`, `StridedAccess`, `ResizableView`, `MultiChannelView`, `MultiChannelViewMut`).
3. Проверить, что все экспортные выражения в `raw-storage/src/lib.rs` и `strided-mem/src/lib.rs` корректно экспортируют добавленные трейты.
4. Выполнить компиляцию без стандартной библиотеки `cargo check --no-default-features --workspace` и со стандартными флагами `cargo test --workspace`.

---

## 8. Требования к безопасности

- **ARCH-001**: Трейты не допускают владения ресурсами, использования `Vec`, `Box` или реализации `Drop`.
- **SAF-001**: `unsafe` локализован исключительно в `VolatileStorage` и сопровождается обязательными секциями `# Safety` в документации.
- **SAF-002**: Мутабельные трейты (`ViewMut`, `ContiguousViewMut`, `ViewComposeMut`, `MultiChannelViewMut`) требуют эксклюзивного заимствования `&mut`.

---

## 9. Требования к горячим путям

- `get_wrapping`, `get_wrapping_mut`, `get_channel_wrapping`, `get_channel_wrapping_mut` — относится к классу `hot-total`. Не возвращают `Option` или `Result`, не паникуют, выполняют прямой просмотр памяти по модулю.
- `len`, `is_empty`, `stride`, `channels`, `as_slices`, `as_slices_mut` — относится к классу `hot-total`.
- `get`, `get_mut`, `get_rel`, `get_rel_mut`, `set_effective_len`, `try_resize` — относится к классу `cold-checked` / `setup`.

---

## 10. Тесты

1. Тесты определения типов: проверить, что существующие структуры `SliceStorage` и `ArrayStorage` реализуют `Storage` и `StorageMut`.
2. Тесты компиляции в режиме `#![no_std]`: `cargo check --no-default-features --workspace`.

---

## 11. Критерий готовности

- Модули `src/traits.rs` крейтов `raw-storage` и `strided-mem` содержат ровно указанные трейты с аннотациями Creusot.
- В модулях `traits.rs` полностью отсутствуют тела функций (кроме стандартных по умолчанию `is_empty`).
- Код компилируется без ошибок через `cargo check --workspace`.

---

## 12. Открытые вопросы

Вопросов нет.
