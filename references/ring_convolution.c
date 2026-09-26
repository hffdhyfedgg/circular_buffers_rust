#include <stdio.h>
#include <stdlib.h>

#include "ring_convolution.h"
#include "_ring_convolution.h"
#include "dema_err.h"
#include "debug.h"
#include "utils.h"
#include "config.h"

/* --- Функции работы с кольцевым буфером --- */

int init_cbuf(CircularBuffer* cbuf, size_t size) {
    // Проверка входных аргументов
    if (!cbuf || size == (size_t)0)
        return DEMA_ERR_INVALID_ARG; // Или другой код ошибки

    // Выделение памяти для внутреннего буфера
    cbuf->buffer = (_flt_type *)calloc(size, sizeof(_flt_type));

    // Проверка выделения памяти для буфера
    if (!cbuf->buffer)
        return DEMA_ERR_MEM_ALLOC;

    // Инициализация полей структуры
    cbuf->size = size;
    cbuf->head = -1; // Или другое начальное значение

    return 0; // Успех
}

CircularBuffer* create_cbuf(size_t size) {

    CircularBuffer* cbuf = (CircularBuffer *)malloc(sizeof(CircularBuffer));

    if(init_cbuf(cbuf, size) != 0) {
        free(cbuf);
        return NULL;
    }
    return cbuf;
}

void free_cbuf(CircularBuffer *cbuf) {
    if (!cbuf)
        return;
    cleanup_cbuf(cbuf);
    free(cbuf);
}

void cleanup_cbuf(CircularBuffer *cbuf) {
    free(cbuf->buffer);
    cbuf->buffer = NULL;
}

void add_to_cbuf(CircularBuffer *cbuf, _flt_type value) {
    // Обновляем голову (последний добавленный элемент)
    cbuf->head = (cbuf->head + 1) % cbuf->size;
    cbuf->buffer[cbuf->head] = value;
}

_flt_type sum_cbuf(const CircularBuffer* cbuf) {
    _flt_type sum = 0.0;
    for (size_t i = 0; i < cbuf->size; i++) {
        sum += GET_CBUF_ELEM(cbuf, i);
    }
    return sum;
}

_flt_type dot_prod_cbuf_on_array(const CircularBuffer* cbuf, const _flt_type* array) {
    _flt_type sum = 0;
    for (size_t i = 0; i < cbuf->size; i++)
        sum+=GET_CBUF_ELEM(cbuf, i)*array[i];
    return sum;
}

_flt_type dot_prod_cbuf_on_cbuf(const CircularBuffer* cbuf1, const CircularBuffer* cbuf2) {
    _flt_type sum = 0;
    for (size_t i = 0; i < cbuf1->size; i++)
        sum += GET_CBUF_ELEM(cbuf1, i)*GET_CBUF_ELEM(cbuf2, i);
    return sum;
}

/* --- Функции работы с банком фильтров --- */

void filter_signal(const CircularBuffer *cbuf, const FilterBank *fb, _flt_type *const results) {
    // Инициализация результатов нулями
    for (int f = 0; f < fb->num_filters; f++) {
        results[f] = 0.0;
    }

    // Перебор отсчётов сигнала
    for (int i = 0; i < fb->filter_length; i++) {
        // int signal_index = (cbuf->head - i + cbuf->size) % cbuf->size;  // Индекс сигнала с учётом "кольцевания"
        _flt_type signal_value = GET_CBUF_ELEM(cbuf, i);

        // Перебор фильтров и обновление сумм
        for (int f = 0; f < fb->num_filters; f++) {
            results[f] += signal_value * fb->filters[f][i];
        }
    }
}

void scale_filter_bank(_flt_type scale_const, FilterBank *fb) {

    for (int f = 0; f < fb->num_filters; f++)
        for (int i = 0; i < fb->filter_length; i++)
            fb->filters[f][i] *= scale_const;
}

/* --- Функции работы со стэком кольцевых буферов --- */

int init_cbuf_stack(CBuffersStack* cbuf_stack, size_t count, size_t size) {
    if (!cbuf_stack || count == 0 || size == 0)
        return DEMA_ERR_INVALID_ARG;

    cbuf_stack->buffers = (_flt_type*)calloc(count*size, sizeof(_flt_type));

    if (!cbuf_stack->buffers)
        return DEMA_ERR_MEM_ALLOC;
    cbuf_stack->count = count;
    cbuf_stack->size = size;
    cbuf_stack->head = (size_t)-1;

    return 0;
}

CBuffersStack* create_cbuf_stack(size_t count, size_t size) {
    CBuffersStack *cbuf_stack = (CBuffersStack *)malloc(sizeof(CBuffersStack));

    if (!cbuf_stack)
        return NULL;

    int result = init_cbuf_stack(cbuf_stack, count, size);
    if (result != 0) {
        free(cbuf_stack->buffers);
        free(cbuf_stack);
        return NULL;
    }

    return cbuf_stack;
}

void free_cbuf_stack(CBuffersStack *cbuf_stack) {
    if (!cbuf_stack)
        return;

    cleanup_cbuf_stack(cbuf_stack);
    free(cbuf_stack);
}

void cleanup_cbuf_stack(CBuffersStack *cbuf_stack) {
    free(cbuf_stack->buffers);
    cbuf_stack->buffers = NULL;

    cbuf_stack->count = 0;
    cbuf_stack->size = 0;
    cbuf_stack->head = (size_t)0;
}

int add_to_cbuf_stack(CBuffersStack *cbuf_stack, const _flt_type *values) {
    if (!cbuf_stack || !values)
        return DEMA_ERR_INVALID_ARG;

    // Обновляем голову (последний добавленный элемент) для всех буферов
    DEBUG_STDOUT("Голова до смещения: %2zu", cbuf_stack->head);
    cbuf_stack->head = (cbuf_stack->head + 1) % cbuf_stack->size;

    DEBUG_STDOUT("Размер буфера: %2zu", cbuf_stack->size);
    DEBUG_STDOUT("Размер стэка: %2zu", cbuf_stack->count);
    DEBUG_STDOUT("Голова: %2zu", cbuf_stack->head);

    // Добавляем соответствующее значение в каждый буфер
    for (size_t i = 0; i < cbuf_stack->count; i++) {
        cbuf_stack->buffers[_CALC_CBUF_STACK_I(i, 0, cbuf_stack->head, cbuf_stack->size)] = values[i];
        DEBUG_STDOUT("Внутренний индекс: %2zu", _CALC_CBUF_STACK_I(i, 0, cbuf_stack->head, cbuf_stack->size));
    }
    return 0;
}

void sum_cbuf_stack(const CBuffersStack* cbuf_stack, _flt_type* sums) {
    for (size_t i = 0; i < cbuf_stack->count; i++) {
        sums[i] = 0.0;
        for (size_t j = 0; j < cbuf_stack->size; j++) {
            sums[i] += GET_CBUF_STACK_ELEM(cbuf_stack, i, j);
        }
    }
}

int get_cbuf_stack_elements(const CBuffersStack *cbuf_stack, ptrdiff_t i, _flt_type* output) {
    if (!cbuf_stack || !cbuf_stack->buffers || !output) // Проверка на NULL
        return DEMA_ERR_INVALID_ARG;

    if (cbuf_stack->count == 0)
        return DEMA_ERR_INVALID_ARG; // Нет буферов

    // Заполняем массив, используя макрос GET_CBUF_STACK_ELEM
    for (size_t bfn = 0; bfn < cbuf_stack->count; bfn++)
        output[bfn] = GET_CBUF_STACK_ELEM(cbuf_stack, bfn, i);

    return 0;
}

void dot_prod_cbuf_stack_on_arrays(
        const CBuffersStack* cbuf_stack,
        size_t arrays_count,
        const _flt_type* arrays,
        _flt_type* output) {

    for (size_t b = 0; b < cbuf_stack->count; b++) {
        for (size_t k = 0; k < arrays_count; k++) {
            output[S2I(b, k, cbuf_stack->count)] = 0;
            for (size_t i = 0; i < cbuf_stack->size; i++)
                output[S2I(b, k, cbuf_stack->count)] += arrays[S2I(k, i, cbuf_stack->size)]*GET_CBUF_STACK_ELEM(cbuf_stack, b, i);
        }
    }
}

void dot_prod_cbuf_stack_on_cbuf_stack(const CBuffersStack* cbuf_stack1, const CBuffersStack* cbuf_stack2, _flt_type* output) {
    for (size_t b1 = 0; b1 < cbuf_stack1->count; b1++) {
        for (size_t b2 = 0; b2 < cbuf_stack2->count; b2++) {
            output[S2I(b1, b2, cbuf_stack2->count)] = 0;
            for (size_t i = 0; i < cbuf_stack1->size; i++)
                output[S2I(b1, b2, cbuf_stack1->count)] += GET_CBUF_STACK_ELEM(cbuf_stack1, b1, i)*GET_CBUF_STACK_ELEM(cbuf_stack2, b2, i);
        }
    }
}


void dot_prod_cbuf_stack_on_arrays_p(
        const CBuffersStack* cbuf_stack,
        size_t arrays_count,
        const _flt_type** arrays,
        _flt_type** output) {
    // Предполагается, что cbuf_stack и arrays корректны
    // и arrays содержит cbuf_stack->count указателей на массивы размером cbuf_stack->size
    for (size_t b = 0; b < cbuf_stack->count; b++) {
        for (size_t k = 0; k < arrays_count; k++) {
            output[b][k] = 0;
            for (size_t i = 0; i < cbuf_stack->size; i++)
                output[b][k] += arrays[k][i]*GET_CBUF_STACK_ELEM(cbuf_stack, b, i);
        }
    }
}

void dot_prod_cbuf_stack_on_cbuf_stack_p(const CBuffersStack* cbuf_stack1, const CBuffersStack* cbuf_stack2, _flt_type** output) {
    for (size_t b1 = 0; b1 < cbuf_stack1->count; b1++) {
        for (size_t b2 = 0; b2 < cbuf_stack2->count; b2++) {
            output[b1][b2] = 0;
            for (size_t i = 0; i < cbuf_stack1->size; i++)
                output[b1][b2] += GET_CBUF_STACK_ELEM(cbuf_stack1, b1, i)*GET_CBUF_STACK_ELEM(cbuf_stack2, b2, i);
        }
    }
}

/* --- Функции для работы с хвостатыми буферами ---*/

int init_cbuf_tail(CBufTail* cbuf_tail, size_t size, size_t tail_len) {
    if (!cbuf_tail || tail_len > size)
        return DEMA_ERR_INVALID_ARG;

    int result = init_cbuf(&(cbuf_tail->buffer), size);
    if (result != 0) return result;

    cbuf_tail->tail_len = tail_len;
    return 0;
}

CBufTail* create_cbuf_tail(size_t size, size_t tail_len) {
    CBufTail* cbuf_tail = (CBufTail*)malloc(sizeof(CBufTail));
    if (!cbuf_tail) return NULL;

    if (init_cbuf_tail(cbuf_tail, size, tail_len) != 0) {
        free(cbuf_tail);
        return NULL;
    }
    return cbuf_tail;
}

void free_cbuf_tail(CBufTail* cbuf_tail) {
    if (!cbuf_tail) return;

    cleanup_cbuf_tail(cbuf_tail);
    free(cbuf_tail);
}

void cleanup_cbuf_tail(CBufTail* cbuf_tail) {
    free(cbuf_tail->buffer.buffer);
    cbuf_tail->buffer.buffer = NULL;
    cbuf_tail->buffer.size = 0;
    cbuf_tail->buffer.head = -1;
}

int resize_cbuf_tail_tail(CBufTail* cbuf_tail, size_t new_tail_len) {
    if (new_tail_len > cbuf_tail->buffer.size)
        return DEMA_ERR_INVALID_ARG;

    cbuf_tail->tail_len = new_tail_len;
    return 0;
}

_flt_type sum_cbuf_tail(const CBufTail* cbuf_tail) {


    _flt_type sum = 0.0;
    size_t effective_size = GET_EFFECTIVE_SIZE_CBUF_TAIL(cbuf_tail);
    for (size_t i = 0; i < effective_size; i++) {
        sum += GET_CBUF_TAIL_ELEM(cbuf_tail, i);
    }
    return sum;
}

_flt_type dot_prod_cbuf_tail_on_array(const CBufTail* cbuf_tail, const _flt_type* array) {
    _flt_type sum = 0;
    size_t effective_size = GET_EFFECTIVE_SIZE_CBUF_TAIL(cbuf_tail);
    for (size_t i = 0; i < effective_size; i++)
        sum += GET_CBUF_TAIL_ELEM(cbuf_tail, i)*array[i];
    return sum;
}

_flt_type dot_prod_cbuf_tail_on_cbuf_tail(const CBufTail* cbuf_tail1, const CBufTail* cbuf_tail2) {
    _flt_type sum = 0;
    size_t effective_size = GET_EFFECTIVE_SIZE_CBUF_TAIL(cbuf_tail1);
    for (size_t i = 0; i < effective_size; i++)
        sum += GET_CBUF_TAIL_ELEM(cbuf_tail1, i)*GET_CBUF_TAIL_ELEM(cbuf_tail2, i);
    return sum;
}

/* --- Функции для работы со стэками хвостатых буферов ---*/

int init_cbuf_stack_tail(CBufStackTail* cbuf_stack_tail, size_t count, size_t size, size_t tail_len) {
    // Проверка входных аргументов
    if (!cbuf_stack_tail || tail_len > size || count == 0 || size == 0)
        return DEMA_ERR_INVALID_ARG;
    int res = init_cbuf_stack(&cbuf_stack_tail->buffers_stack, count, size);
    if(res != 0)
        return res;
    // Устанавливаем длину хвоста
    cbuf_stack_tail->tail_len = tail_len;
    return 0; // Успех
}

CBufStackTail* create_cbuf_stack_tail(size_t count, size_t size, size_t tail_len) {
    // Выделяем память под структуру CBufStackTail
    CBufStackTail* cbuf_stack_tail = (CBufStackTail*)malloc(sizeof(CBufStackTail));
    if (!cbuf_stack_tail)
        return NULL; // Ошибка выделения памяти

    // Инициализируем структуру
    int result = init_cbuf_stack_tail(cbuf_stack_tail, count, size, tail_len);
    if (result != 0) {
        free(cbuf_stack_tail->buffers_stack.buffers);
        // Если инициализация не удалась, освобождаем память под структуру
        free(cbuf_stack_tail);
        return NULL;
    }

    return cbuf_stack_tail; // Возвращаем указатель на созданную структуру
}

void free_cbuf_stack_tail(CBufStackTail *cbuf_stack_tail) {
    // Проверка на NULL
    if (!cbuf_stack_tail)
        return;
    cleanup_cbuf_stack_tail(cbuf_stack_tail);
    // Освобождаем память под саму структуру
    free(cbuf_stack_tail);
}

void cleanup_cbuf_stack_tail(CBufStackTail *cbuf_stack_tail) {
    if (cbuf_stack_tail->buffers_stack.buffers) {
        free(cbuf_stack_tail->buffers_stack.buffers);
        cbuf_stack_tail->buffers_stack.buffers = NULL; // Защита от повторного освобождения
        cbuf_stack_tail->buffers_stack.size = 0;
        cbuf_stack_tail->buffers_stack.head = 0;
        cbuf_stack_tail->buffers_stack.count = 0;
    }
    cbuf_stack_tail->tail_len = 0;
}


int copy_cbuf_stack_tail_elements_to_buffer(const CBufStackTail *cbuf_stack_tail, ptrdiff_t index, _flt_type* output) {
    // Проверка на NULL
    if (!cbuf_stack_tail || !output)
        return DEMA_ERR_INVALID_ARG;

    for(size_t i = 0; i < cbuf_stack_tail->buffers_stack.count; i++)
        output[i] = GET_CBUF_STACK_TAIL_ELEM(cbuf_stack_tail, i, index);

    return 0;
}

int resize_cbuf_stack_tail_tail(CBufStackTail* cbuf_stack_tail, size_t new_tail_len) {
    if (!cbuf_stack_tail)
        return DEMA_ERR_INVALID_ARG;

    // Проверяем, что новая длина хвоста не превышает размер буфера
    if (new_tail_len > cbuf_stack_tail->buffers_stack.size)
        return DEMA_ERR_INVALID_ARG;

    cbuf_stack_tail->tail_len = new_tail_len;
    return 0;
}

void sum_cbuf_stack_tail(const CBufStackTail* cbuf_stack_tail, _flt_type* sums) {
    size_t effective_size = GET_EFFECTIVE_SIZE_CBUF_STACK_TAIL(cbuf_stack_tail);
    for (size_t i = 0; i < cbuf_stack_tail->buffers_stack.count; i++) {
        sums[i] = 0.0;
        for (size_t j = 0; j < effective_size; j++) {
            sums[i] += GET_CBUF_STACK_TAIL_ELEM(cbuf_stack_tail, i, j);
        }
    }
}

void dot_prod_cbuf_stack_tail_on_arrays(
        const CBufStackTail* cbuf_stack_tail,
        size_t arrays_count,
        const _flt_type* arrays,
        _flt_type* output) {
    DEBUG_LOCATION();

    size_t eff_sz = GET_EFFECTIVE_SIZE_CBUF_STACK_TAIL(cbuf_stack_tail);
    DEBUG_CODE(
        PRINTF_S("%20s | %5s | %5s | %9s | %5s | %5s | %9s | %5s\n", " ", "Bfn", "Ind", "Vl", "Arn", "Ai", "Av", "S2I");
    );


    for (size_t b = 0; b < cbuf_stack_tail->buffers_stack.count; b++) {
        for(size_t a = 0; a < arrays_count; a++) {
            output[S2I(b, a, arrays_count)] = 0;
            for(size_t i = 0; i < eff_sz; i++) {
                output[S2I(b, a, cbuf_stack_tail->buffers_stack.count)] += GET_CBUF_STACK_TAIL_ELEM(cbuf_stack_tail, b, i)*arrays[S2I(a, i, eff_sz)];
                DEBUG_CODE(
                    PRINTF_S("%20s | %5zu | %5zu | %3.2e | %5zu | %5zu | %3.2e | %5zu\n", " ", b, i, GET_CBUF_STACK_TAIL_ELEM(cbuf_stack_tail, b, i), a, i, arrays[S2I(a, i, eff_sz)], S2I(a, i, eff_sz));
                );
            }
        }
    }
}

void dot_prod_cbuf_stack_tail_on_cbuf_stack_tail(const CBufStackTail* cbuf_stack_tail1, const CBufStackTail* cbuf_stack_tail2, _flt_type* output) {
    for (size_t b1 = 0; b1 < cbuf_stack_tail1->buffers_stack.count; b1++) {
        for (size_t b2 = 0; b2 < cbuf_stack_tail2->buffers_stack.count; b2++) {
            output[S2I(b1, b2, cbuf_stack_tail2->buffers_stack.count)] = 0;
            for (size_t i = 0; i < cbuf_stack_tail1->buffers_stack.size; i++)
                output[S2I(b1, b2, cbuf_stack_tail1->buffers_stack.count)] += GET_CBUF_STACK_TAIL_ELEM(cbuf_stack_tail1, b1, i)*GET_CBUF_STACK_TAIL_ELEM(cbuf_stack_tail2, b2, i);
        }
    }
}

void dot_prod_cbuf_stack_tail_on_arrays_p(
        const CBufStackTail* cbuf_stack_tail,
        size_t arrays_count,
        const _flt_type** arrays,
        _flt_type** output) {
    size_t effective_size = GET_EFFECTIVE_SIZE_CBUF_STACK_TAIL(cbuf_stack_tail);
    for (size_t b = 0; b < cbuf_stack_tail->buffers_stack.count; b++) {
        for(size_t a = 0; a < arrays_count; a++) {
            output[b][a] = 0;
            for(size_t i = 0; i < cbuf_stack_tail->buffers_stack.size; i++)
                output[b][a] += GET_CBUF_STACK_TAIL_ELEM(cbuf_stack_tail, b, i)*arrays[a][i];
        }
    }
}

void dot_prod_cbuf_stack_tail_on_cbuf_stack_tail_p(const CBufStackTail* cbuf_stack_tail1, const CBufStackTail* cbuf_stack_tail2, _flt_type** output) {
    size_t effective_size = GET_EFFECTIVE_SIZE_CBUF_STACK_TAIL(cbuf_stack_tail1);
    for (size_t b1 = 0; b1 < cbuf_stack_tail1->buffers_stack.count; b1++) {
        for (size_t b2 = 0; b2 < cbuf_stack_tail2->buffers_stack.count; b2++) {
            output[b1][b2] = 0;
            for (size_t i = 0; i < cbuf_stack_tail1->buffers_stack.size; i++)
                output[b1][b2] += GET_CBUF_STACK_TAIL_ELEM(cbuf_stack_tail1, b1, i)*GET_CBUF_STACK_TAIL_ELEM(cbuf_stack_tail2, b2, i);
        }
    }
}
