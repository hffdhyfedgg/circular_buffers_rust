#ifndef UTILS_H
#define UTILS_H

#include "ring_convolution.h"
#include "config.h"




// Структура для хранения параметров populate_filter_bank
typedef struct PopulateFilterBankParams {
    const _flt_type *d_array;  // Массив значений d
    const _flt_type *d_xi_array;  // Массив значений d_xi
    int n;                  // Размер окна детектирования в отсчётах
    int num_elements;       // Количество элементов в массивах d_array и d_xi_array
} PopulateFilterBankParams ;

// Заполнение банка фильтров базисными функциями
void populate_filter_bank(const PopulateFilterBankParams *params, FilterBank *fb);

// Создание заполненного базисными функциями банка фильтров
FilterBank* create_populated_filter_bank(const PopulateFilterBankParams *params);

// Макрос для выделения памяти под двумерный массив.
#define CALLOC_2D_DOUBLE_P(double_pp_var, sz1, sz2) \
    do { \
        double_pp_var = (_flt_type**)calloc(sz1, sizeof(_flt_type*)); \
        for(size_t i = 0; i < sz1; i++) \
            double_pp_var[i] = (_flt_type*)calloc(sz2, sizeof(_flt_type)); \
    } while(0)

#define FREE_2D_DOUBLE_P(double_pp_var, sz1) \
    do { \
        for(size_t i = 0; i < sz1; i++) \
            free(double_pp_var[i]); \
        free(double_pp_var); \
    } while(0)



/* ========== *\
 * Математика *
\* ========== */


/* --- Фабрики --- */

/**
 * @brief Фабрика. Создаёт функцию выбора -- находит элемент, наиболее релевантный
 * согласно функции сравнения cmp_fn.
 *
 * @param `get_expr`: выражение, обязанное принимать collection и индекс, для доступа
 * к элементу. На пример быть `collection[_i]` или `fn_get(collection)`. Для `get_expr` гарантирован
 * доступ ТОЛЬКО к индексу `_i`, коллекции `collection` и внешним данным.
 *
 * @param `cmp_expr`: выражение сравнения. Если возвращает true, то в `_most_similar_idx` пишется
 * новое значение полученное на итерации. Может быть выражением -- простым или вызовом функции:
 * `_most_similar_idx <= _value`,`difficult_compare_fn(_most_similar, _value, ctx_variable)`.
 * Выражение имеет доступ к следующим переменным: `_i`, `_most_similar`, `_most_similar_idx`,`_value`.
 *
 * @info Так-же можно передавать внешний контекст.
 *
 * @return `size_t _most_similar_idx`: индекс выбранного элемента.
 */
#define DEFINE_SELECT(select_fn_name, collection_type, element_type, ctx_type, get_expr, cmp_expr)  \
  size_t select_fn_name(collection_type collection, size_t size, ctx_type ctx_value) {              \
    size_t _i = 0;                                                                                  \
    element_type _most_similar = (get_expr);                                                        \
    size_t _most_similar_idx = 0;                                                                   \
    for (_i = 1; _i < size; _i++) {                                                                 \
      element_type _value = (get_expr);                                                             \
      if (cmp_expr) {                                                                               \
        _most_similar = _value;                                                                     \
        _most_similar_idx = _i;                                                                     \
      }                                                                                             \
    }                                                                                               \
    return _most_similar_idx;                                                                       \
  }

/**
 * @brief Аккумулятор. Использует operation на каждой итерации, записывая результат в `acc`.
 * Значения аккумулируются в `acc`, `acc` имеет тип `output_type`.
 *
 * @warning В `operation` можно передать вызов функции-аккумулции с сигнатурой: `output_type func(output_type acc, element_type value)`.
 * Для сохранения производительности, функция `func` обязана быть определена с атрибутами `static inline` и быть в том же файле, где
 * используется `DEFINE_ACCUMULATE` с ней.
 * Пример:
 * ```c
 * int get(const int* arr, size_t idx) {
 *  return arr[idx];
 * }
 *
 * static inline int sum(int x, int y) {
 *  return x + y;
 * }
 *
 * DEFINE_ACCUMULATE(acc_sum, const int*, int, int, get, sum(_acc,_x))
 * ```
 *  В поле `operation` функция аккумуляции обязана принимать только аргументы `_acc` и `_x`, где `_acc` аккумулированная величина
 * за предыдущие итерации, а `_x` полученное с помощью функции переданной в поле `getter` из массива.
 *
 */
#define DEFINE_ACCUMULATE(accumulate_fn_name, collection_type, element_type, output_type, getter, operation)  \
  output_type accumulate_fn_name(collection_type collection, size_t size, output_type init_value) {           \
    output_type _acc = init_value;                                                                            \
    for(size_t i = 0; i < size; i++) {                                                                        \
      _x = getter(collection, i);                                                                             \
      _acc = (operation);                                                                                     \
    }                                                                                                         \
    return _acc;                                                                                              \
  }


/* --- Макросы --- */


/* --- Функции --- */

/**
 * Поиск индекса максимума.
 */
size_t argmax(const _flt_type* array, size_t size);

/**
 * Поиск индекса минимума.
 */
size_t argmin(const _flt_type* array, size_t size);



/**
 * Применить функцию fn к каждому элементу source и записать результат в соответствующий
 * элемент массива target.
 *
 * Можно передать произвольный набор байт через void *ctx, который будет передан в fn на каждой итерации.
 */
void foreach_from_flt_to_flt_ctx(
  const _flt_type* source,
  _flt_type* target,
  const size_t size,
  _flt_type (*fn)(_flt_type, void*),
  void* ctx
);

/**
 * Применить функцию fn к каждому элементу source и записать результат в соответствующий
 * элемент массива target.
 *
 * Можно передать произвольный набор байт через const void *ctx, который будет передаваться в fn на каждой итерации.
 *
 * ctx -- константный указатель!
 */
void foreach_from_flt_to_flt_const_ctx(
  const _flt_type* source,
  _flt_type* target,
  const size_t size,
  _flt_type (*fn)(_flt_type, const void*),
  const void* ctx
);

/**
 * Применить функцию fn к каждому элементу array и записать результат в соответствующий
 * элемент массива array.
 *
 * Можно передать произвольный набор байт через void *ctx, который будет передан в fn на каждой итерации.
 */
void foreach_flt_ctx(
  _flt_type* array,
  const size_t size,
  _flt_type (*fn)(_flt_type, void*),
  void* ctx
);

/**
 * Применить функцию fn к каждому элементу array и записать результат в соответствующий
 * элемент массива array.
 *
 * Можно передать произвольный набор байт через const void *ctx, который будет передаваться в fn на каждой итерации.
 *
 * ctx -- константный указатель!
 */
void foreach_flt_const_ctx(
  _flt_type* array,
  const size_t size,
  _flt_type (*fn)(_flt_type, const void*),
  const void* ctx
);

/**
 * Применяет функцию fn к каждому элементу массива array, аккумулируя и возвращая результат.
 * Можно передать произвольный набор байт через void *ctx, который будет передаваться в fn на каждой итерации.
 *
 * Начальное значение необходимо указать в init_value.
 *
 * Сигнатура модификатора:
 *
 * _flt_type fn(_flt_type accumulator, _flt_type element_of_array, void* context)
 * где:
 *
 * - accumulator: на первой итерации принимает init_value, на последующих то, что вернула функция на прошлой итерации.
 *
 * - element_of_array: элемент массива на новой итерации
 *
 * - context: переданный контекст.
 */
static inline _flt_type accumulate_flt_ctx(
  _flt_type* array,
  const size_t size,
  _flt_type init_value,
  _flt_type (*fn)(_flt_type, _flt_type, void*),
  void* ctx
) {
  _flt_type result = init_value;
  for(size_t i = 0; i < size; i++) {
    result = fn(result, array[i], ctx);
  }
  return result;
}

/* === */


/**
 * @brief Макрос для выделения памяти под двумерный массив,
 * основанный на линейном массиве.
 *
 * @note Для индексации необходимо использовать
 *
 * @ref `S2I`
 */

/**
 * @brief Макрос для перевода 2D индекса в линейный (Аналог sub2ind в матлаб).
 *
 * Обеспечивает работу с обычными массивами (_flt_type*, float*, int* и т.д.)
 * как с двумерными.
 *
 * @param i1 Номер строки (массива).
 * @param i2 Номер колонки (эллемента в массиве).
 * @param sz Длинна строки.
 *
 * @warning Не проверяет выход за границы массива!
 * Не правильные индексы могут привести к обращению в область памяти вне массива.
 */
#define S2I(i1, i2, sz) i1*sz + i2

#endif // UTILS_H
