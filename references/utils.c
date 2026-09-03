#include "utils.h"
#include "ring_convolution.h"
#include <stdlib.h>
#include <basis.h>

// Функция для заполнения структуры FilterBank базисными функциями
void populate_filter_bank(const PopulateFilterBankParams *params, FilterBank *fb) {
    // Инициализация банка фильтров
    fb->num_filters = params->num_elements * 3;  // Три базисные функции для каждого набора параметров
    fb->filters = (_flt_type **)malloc(fb->num_filters * sizeof(_flt_type *));
    fb->filter_length = params->n;  // РАзмер фильтров (длинна)

    int filter_index = 0;

    // Перебор всех элементов массивов
    for (int i = 0; i < params->num_elements; i++) {
      _flt_type d = params->d_array[i];
      _flt_type d_xi = params->d_xi_array[i];
      int n = params->n;

      // Вычисление базисных функций
      _flt_type *f1, *f2, *f3;

      calc_basis(d, d_xi, n, &f1, &f2, &f3);

      // Сохранение базисных функций в банке фильтров
      fb->filters[filter_index++] = f1;
      fb->filters[filter_index++] = f2;
      fb->filters[filter_index++] = f3;
    }
}


// Создание заполненного базисными функциями банка фильтров
FilterBank* create_populated_filter_bank(const PopulateFilterBankParams *params) {
	FilterBank *fb = (FilterBank *)malloc(sizeof(FilterBank));
	populate_filter_bank(params, fb);
	return fb;
}

size_t argmax(const _flt_type* array, size_t size) {
  size_t ind_max = 0;
  size_t current_max = _FLT_N_MIN;
  for(size_t i = 0; i < size; i++) {
    _flt_type new_flt = array[i];
    if (new_flt > current_max) {
      current_max = new_flt;
      ind_max = i;
    }
  }
  return ind_max;
}

size_t argmin(const _flt_type* array, size_t size) {
  size_t ind_min = 0;
  size_t current_min = _FLT_MAX;
  for(size_t i = 0; i < size; i++) {
    _flt_type new_flt = array[i];
    if (new_flt < current_min) {
      current_min = new_flt;
      ind_min = i;
    }
  }
  return ind_min;
}

void foreach_from_flt_to_flt_ctx(
  const _flt_type* source,
  _flt_type* target,
  const size_t size,
  _flt_type (*fn)(_flt_type, void*),
  void* ctx
) {
  for (size_t i = 0; i < size; i++) {
    target[i] = fn(source[i], ctx);
  }
}

void foreach_from_flt_to_flt_const_ctx(
  const _flt_type* source,
  _flt_type* target,
  const size_t size,
  _flt_type (*fn)(_flt_type, const void*),
  const void* ctx
) {
  for (size_t i = 0; i < size; i++) {
    target[i] = fn(source[i], ctx);
  }
}

void foreach_flt_ctx(
  _flt_type* array,
  const size_t size,
  _flt_type (*fn)(_flt_type, void*),
  void* ctx
) {
  foreach_from_flt_to_flt_ctx(array, array, size, fn, ctx);
}

void foreach_flt_const_ctx(
  _flt_type* array,
  const size_t size,
  _flt_type (*fn)(_flt_type, const void*),
  const void* ctx
) {
  foreach_from_flt_to_flt_const_ctx(array, array, size, fn, ctx);
}

_flt_type cumulative_flt_ctx(
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
