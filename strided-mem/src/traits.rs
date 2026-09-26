
/// Базовый контракт неизменяемого представления.
///
/// Индекс `index` — это ЛОГИЧЕСКИЙ индекс внутри данного представления,
/// начиная с 0. Представление само отвечает за преобразование
/// логического индекса в физический доступ к памяти.
pub trait View<'a, Idx> {
    type Item;

    /// Число элементов, доступных через данное представление.
    fn len(&self) -> usize;

    /// Возвращает `true`, если `self.len() == 0`.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Доступ к элементу по логическому индексу.
    /// Возвращает `None`, если `index >= self.len()`.
    fn try_get(&self, index: usize) -> Option<&'a Self::Item>;

    /// Доступ к элементу по логическому индексу.
    fn get(&self, index: usize) -> &'a Self::Item;
}

/// Контракт мутабельного представления.
pub trait ViewMut<'a>: View<'a> {
    /// Мутабельный доступ к элементу по логическому индексу.
    /// Возвращает `None`, если `index >= self.len()`.
    fn try_get_mut(&mut self, index: usize) -> Option<&mut Self::Item>;

    /// Мутабельный доступ к элементу по логическому индексу.
    fn get_mut(&mut self, index: usize) -> &mut Self::Item;
}

/// Представление, данные которого лежат в одном или двух
/// непрерывных фрагментах памяти (кольцевой буфер).
pub trait ContiguousView<'a>: View<'a> {
    /// Возвращает данные в логическом порядке как
    /// не более чем два непрерывных среза.
    /// Если данные не «заворачиваются», второй срез пуст.
    fn as_slices(&self) -> (&'a [Self::Item], &'a [Self::Item]);
}

/// Мутабельный аналог `ContiguousView`.
pub trait ContiguousViewMut<'a>: ViewMut<'a> {
    fn as_slices_mut(&mut self) -> (&mut [Self::Item], &mut [Self::Item]);
}
